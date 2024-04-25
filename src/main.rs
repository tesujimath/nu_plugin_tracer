use anyhow::{anyhow, Context, Result};
use home::home_dir;
use std::{
    env::{args_os, ArgsOs},
    ffi::{OsStr, OsString},
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

fn get_plugin_from_args() -> Result<(OsString, PathBuf, ArgsOs)> {
    let mut args = args_os();
    let path: PathBuf = args.nth(1).ok_or(anyhow!("missing plugin path"))?.into();

    let name = path
        .file_name()
        .ok_or(anyhow!("failed to determine file name for plugin"))?
        .to_os_string();

    Ok((name, path, args))
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

async fn open_trace_file<P, S>(home: P, plugin_name: S, suffix: &str) -> anyhow::Result<File>
where
    P: AsRef<Path>,
    S: AsRef<OsStr>,
{
    let mut suffixed_name = plugin_name.as_ref().to_os_string();
    suffixed_name.push(suffix);
    let path = home.as_ref().join(suffixed_name);
    let f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    Ok(f)
}

async fn trace_plugin() -> anyhow::Result<()> {
    let home = home_dir().ok_or(anyhow!("can't determine user home directory"))?;
    let (plugin_name, plugin_path, plugin_args) = get_plugin_from_args()?;

    let stdin = stdin();
    let stdout = stdout();
    pin!(stdin);
    pin!(stdout);

    let mut plugin = Command::new(&plugin_path)
        .args(plugin_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "can't execute wrapped plugin {}",
                &plugin_path.to_string_lossy(),
            )
        })?;

    let (plugin_stdin, plugin_stdout) =
        (plugin.stdin.take().unwrap(), plugin.stdout.take().unwrap());
    pin!(plugin_stdin);
    pin!(plugin_stdout);

    let raw_in = open_trace_file(&home, &plugin_name, ".in.raw").await?;
    pin!(raw_in);

    let raw_out = open_trace_file(&home, &plugin_name, ".out.raw").await?;
    pin!(raw_out);

    tokio::select!(
        _ = forward(stdin, plugin_stdin, raw_in) => {
        },
        _ = forward(plugin_stdout, stdout, raw_out) => {
        },
        _ = plugin.wait() => {
        }
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
