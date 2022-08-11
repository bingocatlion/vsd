<h1 align="center">vsd</h1>

<p align="center">
  <a href="https://github.com/clitic/vsd">
    <img src="https://img.shields.io/github/downloads/clitic/vsd/total?logo=github&style=flat-square">
  </a>
  <a href="https://crates.io/crates/vsd">
    <img src="https://img.shields.io/crates/d/vsd?logo=rust&style=flat-square">
  </a>
  <a href="https://crates.io/crates/vsd">
    <img src="https://img.shields.io/crates/v/vsd?style=flat-square">
  </a>
  <a href="https://docs.rs/vsd/latest/vsd">
    <img src="https://img.shields.io/docsrs/vsd?logo=docsdotrs&style=flat-square">
  </a>
  <a href="https://github.com/clitic/vsd">
    <img src="https://img.shields.io/github/license/clitic/vsd?style=flat-square">
  </a>
  <a href="https://github.com/clitic/vsd">
    <img src="https://img.shields.io/github/repo-size/clitic/vsd?logo=github&style=flat-square">
  </a>
  <a href="https://github.com/clitic/vsd">
    <img src="https://img.shields.io/tokei/lines/github/clitic/vsd?style=flat-square">
  </a>
</p>

<p align="center">
  <a href="#Installations">Installations</a>
  &nbsp;&nbsp;&nbsp;|&nbsp;&nbsp;&nbsp;
  <a href="#Usage">Usage</a>
</p>

Command line program to download HLS video from websites and m3u8 links.

Know more about HLS from [howvideo.works](https://howvideo.works) and 
[wikipedia](https://en.wikipedia.org/wiki/M3U).

There are some alternatives to vsd but they lack in some features like [N_m3u8DL-CLI](https://github.com/nilaoda/N_m3u8DL-CLI) is not cross platform and [m3u8-downloader](https://github.com/llychao/m3u8-downloader) has very few customizable options. There are also options like [webvideo-downloader](https://github.com/jaysonlong/webvideo-downloader) which open websites using chrome and captures the m3u8 links and then download it. A similar functionality can achieved with vsd too by using *capture* and *collect* features. 

<p align="center">
  <img src="https://github.com/clitic/vsd/blob/main/images/showcase.png">
</p>

## Features

- [x] Captures m3u8 network requests from websites.
- [x] Collects .m3u8, .mpd and subtitles from websites and save them locally.
- [x] Custom headers, proxies and cookies.
- [x] Inbuilt web scrapper for querying HLS and DASH links.
- [x] Human friendly resolution and bandwidth based master playlist variants parsing.
- [x] Multiple output formats which are supported by ffmpeg.
- [x] Mux seperate video, audio and subtitle (webvtt) stream to a single file.
- [x] Progressive binary merging of segments.
- [x] Realtime file size estimation.
- [x] Select standard resolution playlists like `HD`, `FHD` etc.
- [x] Supports `AES-128` playlist decryption.
- [x] Supports downloading in multiple threads.
- [x] Supports resume and retries.
- [ ] GUI
- [ ] Supports Dash
- [ ] Supports [SAMPLE-AES](https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.2.4) playlist decryption.
- [ ] Supports live stream download.

<a href="#Help">See More</a>

## Installations

Dependencies

- [ffmpeg](https://www.ffmpeg.org/download.html) (optional) only required for transmuxing and transcoding streams.
- [chrome](https://www.google.com/chrome) / [chromium](https://www.chromium.org/getting-involved/download-chromium/) (optional) only required for `CHROME OPTIONS` related flag. 

Visit [releases](https://github.com/clitic/vsd/releases) for prebuilt binaries. You just need to copy that binary to any path specified in your `PATH` environment variable.

### Through Cargo

```bash
cargo install vsd
```

### On x86_64 Linux

```bash
$ wget https://github.com/clitic/vsd/releases/download/v0.1.2/vsd-v0.1.2-x86_64-unknown-linux-musl.tar.gz -O vsd-v0.1.2.tar.gz
$ tar -xzf vsd-v0.1.2.tar.gz -C /usr/local/bin/
$ chmod +x /usr/local/bin/vsd
$ rm vsd-v0.1.2.tar.gz
```

### On Termux

Android builds are compiled with **android-ndk-r22b** and targets **API Level 30** it means binary is only supported by **Android 11** and above. Also, see [running on android](https://github.com/clitic/vsd/blob/main/docs/running-on-android.md).

```bash
$ pkg install wget ffmpeg
$ wget https://github.com/clitic/vsd/releases/download/v0.1.2/vsd-v0.1.2-aarch64-linux-android.tar.gz -O vsd-v0.1.2.tar.gz
$ tar -xzf vsd-v0.1.2.tar.gz -C $PREFIX/bin/
$ chmod +x $PREFIX/bin/vsd
$ rm vsd-v0.1.2.tar.gz
```

## Usage

For quick testing purposes you may use [https://test-streams.mux.dev](https://test-streams.mux.dev) as direct input. These streams are used by [hls.js](https://github.com/video-dev/hls.js) for testing purposes.

- Downloading HLS video from a website, m3u8 url or from a local m3u8 file.

```bash
$ vsd <url | .m3u8> -o video.mp4
```

- Collecting .m3u8 (HLS), .mpd (Dash) and subtitles from a website and saving them locally.

```bash
$ vsd <url> --collect
```

## Help

```bash
$ vsd --help
```

```
vsd 0.1.2
clitic <clitic21@gmail.com>
Command line program to download HLS video from websites and m3u8 links.

USAGE:
    vsd.exe [OPTIONS] <INPUT>

ARGS:
    <INPUT>    url | .m3u8 | .m3u

OPTIONS:
    -a, --alternative                  Download alternative streams such as audio and subtitles
                                       streams from master playlist instead of variant video streams
    -b, --baseurl <BASEURL>            Base url for all segments. Usually needed for local m3u8 file
    -h, --help                         Print help information
    -o, --output <OUTPUT>              Path of final downloaded video stream. For file extension any
                                       ffmpeg supported format could be provided. If playlist
                                       contains alternative streams vsd will try to transmux and
                                       trancode into single file using ffmpeg
    -q, --quality <QUALITY>            Automatic selection of some standard resolution streams with
                                       highest bandwidth stream variant from master playlist
                                       [default: select] [possible values: select, sd, hd, fhd, uhd,
                                       uhd4k, max]
    -r, --resume                       Resume a download session. Download session can only be
                                       resumed if download session json file is present
        --raw-prompts                  Raw style input prompts for old and unsupported terminals
        --retry-count <RETRY_COUNT>    Maximum number of retries to download an individual segment
                                       [default: 15]
    -s, --skip                         Skip downloading and muxing alternative streams
    -t, --threads <THREADS>            Maximum number of threads for parllel downloading of
                                       segments. Number of threads should be in range 1-16
                                       (inclusive) [default: 5]
    -V, --version                      Print version information

CHROME OPTIONS:
        --build       Build http links for all uri present in .m3u8 file while collecting it.
                      Resultant .m3u8 file can be played and downloaded directly without the need of
                      `--baseurl` flag. This option should must be used with `--collect` flag only
        --capture     Launch Google Chrome to capture requests made to fetch .m3u8 (HLS) and .mpd
                      (Dash) files
        --collect     Launch Google Chrome and collect .m3u8 (HLS), .mpd (Dash) and subtitles from a
                      website and save them locally
        --headless    Launch Google Chrome without a window for interaction. This option should must
                      be used with `--capture` or `--collect` flag only

CLIENT OPTIONS:
        --cookies <cookies> <url>
            Enable cookie store and fill it with some existing cookies. Example `--cookies "foo=bar;
            Domain=yolo.local" https://yolo.local`. This option can be used multiple times

        --enable-cookies
            Enable cookie store which allows cookies to be stored

        --header <key> <value>
            Custom headers for requests. This option can be used multiple times

        --proxy-address <PROXY_ADDRESS>
            Set http or https proxy address for requests

        --user-agent <USER_AGENT>
            Update and set custom user agent for requests [default: "Mozilla/5.0 (Windows NT 10.0;
            Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.64 Safari/537.36"]
```

## Building From Source

1. Install [Rust](https://www.rust-lang.org)

2. Download or clone Repository.

```bash
git clone https://github.com/clitic/vsd.git
```

3. Build Release (inside vsd directory)

### Linux

First install [openssl](https://docs.rs/openssl/latest/openssl/#automatic) library then run.

```bash
OPENSSL_STATIC=true cargo build --release
```

### Windows

Build [openssl](https://github.com/openssl/openssl) library or download and install it from [Win32OpenSSL](https://slproweb.com/products/Win32OpenSSL.html). vsd builds use openssl static build i.e. [openssl-3.0.5-VC-WIN64A-static.7z](https://drive.google.com/file/d/1LhVu97TiV4HSzxUH-rjiGXXZBs27iDbs/view?usp=sharing).

```powershell
$env:OPENSSL_DIR="C:\openssl-3.0.5-VC-WIN64A-static"
$env:OPENSSL_STATIC=$true
cargo build --release
```

### Android (On Linux 64-bit)

1. Install [NDK](https://developer.android.com/ndk/downloads).

```bash
$ wget https://dl.google.com/android/repository/android-ndk-r22b-linux-x86_64.zip
$ unzip android-ndk-r22b-linux-x86_64.zip
$ rm android-ndk-r22b-linux-x86_64.zip
```

2. Add android target aarch64-linux-android.

```bash
$ rustup target add aarch64-linux-android
```

3. Add linker path to `~/.cargo/config.toml` file.

```toml
[target.aarch64-linux-android]
linker = "android-ndk-r22b/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"
```

4. Now compile with target aarch64-linux-android.

```bash
$ PATH=android-ndk-r22b/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
$ cargo build --release --target aarch64-linux-android
```

## License

Dual Licensed

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([LICENSE-APACHE](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([LICENSE-MIT](LICENSE-MIT))
