use anyhow::{anyhow, Result};
use home::home_dir;
use std::{
    env::args_os,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
};
use tokio::{
    fs::OpenOptions,
    io::{stdin, stdout, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    pin,
    process::Command,
};

fn program_name() -> Result<String> {
    args_os()
        .next()
        .ok_or(anyhow!("can't find own program name"))
        .and_then(|s| {
            Ok(Path::new(&s)
                .file_name()
                .ok_or(anyhow!("failed to determine file_name for program"))?
                .to_str()
                .ok_or(anyhow!("program name is not valid unicode"))?
                .to_string())
        })
}

fn default_plugins_dir() -> Result<PathBuf> {
    let config_dir: PathBuf = nu_path::config_dir().ok_or(anyhow!("can't find Nu config dir"))?;

    Ok(config_dir.join("nushell").join("plugins"))
}

async fn forward<R, W, Tee>(
    mut r: Pin<&mut R>,
    mut w: Pin<&mut W>,
    mut tee: Pin<&mut Tee>,
) -> std::io::Result<()>
where
    R: AsyncRead,
    W: AsyncWrite,
    Tee: AsyncWrite,
{
    let mut done = false;
    let mut buf: [u8; 1024] = [0; 1024];

    while !done {
        let n_read = r.read(&mut buf).await?;
        if n_read == 0 {
            done = true;
        } else {
            w.write_all(&buf[..n_read]).await?;
            w.flush().await?;
            tee.write_all(&buf[..n_read]).await?;
            tee.flush().await?;
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let home = home_dir().ok_or(anyhow!("can't determine user home directory"))?;
    let appender = tracing_appender::rolling::never(home, "nu_plugin_tracer.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::fmt().with_writer(non_blocking_appender);
    tracing::subscriber::set_global_default(subscriber.finish())?;

    let tracer_name = program_name()?;
    let suffix = "_tracer";

    let plugin_name = tracer_name
        .strip_suffix("_tracer")
        .ok_or(anyhow!("program name doesn't end with {}", suffix))?;
    let plugin_path = default_plugins_dir().map(|p| p.join(plugin_name))?;

    let mut plugin = Command::new(&plugin_path)
        .args(std::env::args().skip(1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let (plugin_stdin, plugin_stdout) =
        (plugin.stdin.take().unwrap(), plugin.stdout.take().unwrap());

    tracing::info!("tracing {:?}", &plugin_path);

    let raw_in_path = PathBuf::from(format!("{}.in.raw", &plugin_name));
    let raw_in = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&raw_in_path)
        .await?;
    pin!(raw_in);
    let raw_out_path = PathBuf::from(format!("{}.out.raw", &plugin_name));
    let raw_out = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&raw_out_path)
        .await?;
    pin!(raw_out);

    let stdin = stdin();
    let stdout = stdout();
    pin!(stdin);
    pin!(stdout);
    pin!(plugin_stdin);
    pin!(plugin_stdout);
    tokio::select!(
        _ = forward(stdin, plugin_stdin, raw_in) => {
            tracing::info!("in tee is done");
        },
        _ = forward(plugin_stdout, stdout, raw_out) => {
            tracing::info!("out tee is done");
        },
        _ = plugin.wait() => {
            tracing::info!("plugin is done");
        }
    );

    tracing::info!("tracer is done");
    Ok(())
}
