use anyhow::{anyhow, Result};
use std::{
    env::args_os,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    time::Duration,
};
use tokio::{
    fs::{File, OpenOptions},
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
    let config_dir: PathBuf = nu_path::config_dir().ok_or(anyhow!("can't find user config dir"))?;

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

async fn open_trace_file(plugin_name: &str, suffix: &str) -> anyhow::Result<File> {
    let path = PathBuf::from(format!("{}{}", plugin_name, suffix));
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    Ok(f)
}

async fn trace_plugin() -> anyhow::Result<()> {
    let tracer_name = program_name()?;
    let tracer_suffix = "_tracer";

    let plugin_name = tracer_name
        .strip_suffix("_tracer")
        .ok_or(anyhow!("program name doesn't end with {}", tracer_suffix))?;
    let plugin_path = default_plugins_dir().map(|p| p.join(plugin_name))?;

    let stdin = stdin();
    let stdout = stdout();
    pin!(stdin);
    pin!(stdout);

    let mut plugin = Command::new(&plugin_path)
        .args(std::env::args().skip(1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let (plugin_stdin, plugin_stdout) =
        (plugin.stdin.take().unwrap(), plugin.stdout.take().unwrap());
    pin!(plugin_stdin);
    pin!(plugin_stdout);

    let raw_in = open_trace_file(plugin_name, ".in.raw").await?;
    pin!(raw_in);

    let raw_out = open_trace_file(plugin_name, ".out.raw").await?;
    pin!(raw_out);

    tokio::select!(
        _ = forward(stdin, plugin_stdin, raw_in) => { },
        _ = forward(plugin_stdout, stdout, raw_out) => { },
        _ = plugin.wait() => { }
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let result = runtime.block_on(trace_plugin());

    // https://github.com/tokio-rs/tokio/issues/2466
    // https://github.com/tokio-rs/tokio/issues/2318#issuecomment-599651871
    runtime.shutdown_timeout(Duration::from_secs(0));

    result
}
