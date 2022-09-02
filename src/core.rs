use crate::merger::BinarySequence;
use crate::progress::{DownloadProgress, StreamData};
use crate::utils;
use crate::{dash, hls};
use crate::{Args, InputType};
use anyhow::{anyhow, bail, Result};
use kdam::prelude::*;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::HeaderValue;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct DownloadState {
    pub args: Args,
    client: Arc<Client>,
    progress: DownloadProgress,
}

impl DownloadState {
    pub fn new(args: Args) -> Result<Self> {
        let client = args.client()?;

        if let Some(output) = &args.output {
            if !output.ends_with(".ts") {
                utils::check_ffmpeg("the given output doesn't have .ts file extension")?
            }
        }

        Ok(Self {
            args,
            client,
            progress: DownloadProgress::new_empty(),
        })
    }

    fn scrape_website(&mut self) -> Result<()> {
        println!("Scraping website for HLS and Dash links.");
        let links =
            crate::utils::find_hls_dash_links(&self.client.get(&self.args.input).send()?.text()?);

        match links.len() {
            0 => bail!(utils::scrape_website_message(&self.args.input)),
            1 => {
                self.args.input = links[0].clone();
                println!("Found one link {}", &links[0]);
            }
            _ => {
                let mut elinks = vec![];
                for (i, link) in links.iter().enumerate() {
                    elinks.push(format!("{:#2}) {}", i + 1, link));
                }
                let index = utils::select(
                    "Select one link:".to_string(),
                    &elinks,
                    self.args.raw_prompts,
                )?;
                self.args.input = links[index].clone();
            }
        }

        if self.args.input.ends_with(".mpd") {
            bail!("Dash streams are not supported.")
        }

        Ok(())
    }

    fn download_alternative(&mut self, master: &m3u8_rs::MasterPlaylist) -> Result<()> {
        let stream_input = self.args.input.clone();
        let audio_tempfile = format!(
            "{}_audio.ts",
            self.progress.stream.file.trim_end_matches(".ts")
        );
        let subtitle_tempfile = format!(
            "{}_subtitles.vtt",
            self.progress.stream.file.trim_end_matches(".ts")
        );
        let subtitle_output = format!(
            "{}_subtitles.srt",
            self.progress.stream.file.trim_end_matches(".ts")
        );

        for alternative in &master.alternatives {
            self.args.input = stream_input.clone();

            match alternative.media_type {
                m3u8_rs::AlternativeMediaType::Audio => {
                    if alternative.autoselect {
                        if let Some(uri) = &alternative.uri {
                            println!("{} audio stream.", "Downloading".colorize("bold green"));
                            utils::check_ffmpeg("audio stream needs to muxed with video stream")?;
                            self.args.input = self.args.get_url(uri)?;
                            self.progress.current("audio");
                            self.progress.audio =
                                Some(StreamData::new(&self.args.input, &audio_tempfile));

                            let content =
                                self.client.get(&self.args.input).send()?.bytes()?.to_vec();

                            if let m3u8_rs::Playlist::MediaPlaylist(meadia) =
                                m3u8_rs::parse_playlist_res(&content).map_err(|_| {
                                    anyhow!("Couldn't parse {} playlist.", self.args.input)
                                })?
                            {
                                self.download(&meadia.segments, audio_tempfile.clone())?;
                            }
                        }
                    }
                }

                m3u8_rs::AlternativeMediaType::Subtitles
                | m3u8_rs::AlternativeMediaType::ClosedCaptions => {
                    if alternative.autoselect {
                        if let Some(uri) = &alternative.uri {
                            println!("{} subtitles stream.", "Downloading".colorize("bold green"));
                            utils::check_ffmpeg(
                                "subtitles stream needs to muxed with video stream",
                            )?;
                            self.args.input = self.args.get_url(uri)?;
                            self.progress.current("subtitle");
                            self.progress.subtitle =
                                Some(StreamData::new(&self.args.input, &subtitle_tempfile));

                            let content =
                                self.client.get(&self.args.input).send()?.bytes()?.to_vec();

                            if let m3u8_rs::Playlist::MediaPlaylist(meadia) =
                                m3u8_rs::parse_playlist_res(&content).map_err(|_| {
                                    anyhow!("Couldn't parse {} playlist.", self.args.input)
                                })?
                            {
                                self.download(&meadia.segments, subtitle_tempfile.clone())?;
                            }

                            if std::path::Path::new(&subtitle_output).exists() {
                                std::fs::remove_file(&subtitle_output)?;
                            }

                            println!(
                                "Executing {}",
                                ["ffmpeg", "-i", &subtitle_tempfile, &subtitle_output]
                                    .join(" ")
                                    .colorize("cyan")
                            );

                            let code = std::process::Command::new("ffmpeg")
                                .args(["-i", &subtitle_tempfile, &subtitle_output])
                                .stderr(std::process::Stdio::null())
                                .spawn()?
                                .wait()?;

                            if !code.success() {
                                bail!("FFMPEG exited with code {}.", code.code().unwrap_or(1))
                            }

                            std::fs::remove_file(&subtitle_tempfile)?;

                            if let Some(subtitle) = &mut self.progress.subtitle {
                                subtitle.file = subtitle_tempfile.clone();
                            }
                        }
                    }
                }

                _ => (),
            }
        }

        self.args.input = stream_input;
        self.progress.current("stream");
        Ok(())
    }

    pub fn segments(&mut self) -> Result<Vec<m3u8_rs::MediaSegment>> {
        if self.args.input_type().is_website() {
            self.scrape_website()?;
        }

        let input_type = self.args.input_type();

        let content = match input_type {
            InputType::HlsUrl | InputType::DashUrl => {
                self.client.get(&self.args.input).send()?.bytes()?.to_vec()
            }
            InputType::HlsLocalFile | InputType::DashLocalFile => {
                std::fs::read_to_string(&self.args.input)?
                    .as_bytes()
                    .to_vec()
            }
            InputType::LocalFile | InputType::Website => bail!("Unsupported input file type."),
        };

        if input_type.is_hls() {
            match m3u8_rs::parse_playlist_res(&content)
                .map_err(|_| anyhow!("Couldn't parse {} playlist.", self.args.input))?
            {
                m3u8_rs::Playlist::MasterPlaylist(master) => {
                    self.args.input = if self.args.alternative {
                        self.args
                            .get_url(&hls::alternative(&master, self.args.raw_prompts)?)?
                    } else {
                        self.args.get_url(&hls::master(
                            &master,
                            &self.args.quality,
                            self.args.raw_prompts,
                        )?)?
                    };

                    self.progress.current("stream");
                    self.progress.stream = StreamData::new(&self.args.input, &self.args.tempfile());
                    self.progress
                        .json_file(&utils::replace_ext(&self.progress.stream.file, "json"));

                    if !self.args.alternative && !self.args.skip {
                        self.download_alternative(&master)?;
                    }

                    match m3u8_rs::parse_playlist_res(
                        &self.client.get(&self.args.input).send()?.bytes()?.to_vec(),
                    )
                    .map_err(|_| anyhow!("Couldn't parse {} playlist.", self.args.input))?
                    {
                        m3u8_rs::Playlist::MediaPlaylist(media) => {
                            if media.end_list {
                                println!(
                                    "{} {} stream.",
                                    "Downloading".colorize("bold green"),
                                    if self.args.alternative {
                                        "alternative"
                                    } else {
                                        "video"
                                    }
                                );
                                return Ok(media.segments);
                            } else {
                                let live_playlist = hls::LivePlaylist::new(
                                    &self.args.input,
                                    self.client.clone(),
                                    self.args.record_duration,
                                );
                                let mut file = std::fs::File::create(&self.args.tempfile())?;
                                let mut pb = tqdm!(
                                    // total = total,
                                    unit = "ts".to_owned(),
                                    dynamic_ncols = true
                                );
                                pb.refresh();
                                let mut total_bytes = 0;

                                for media in live_playlist {
                                    for seg in media.map_err(|x| anyhow!(x))?.segments {
                                        let bytes = self
                                            .client
                                            .get(&self.args.get_url(&seg.uri)?)
                                            .send()?
                                            .bytes()?
                                            .to_vec();
                                        total_bytes += bytes.len();
                                        file.write_all(&bytes)?;
                                        pb.set_description(utils::format_bytes(total_bytes, 2).2);
                                        pb.update(1);
                                    }
                                }

                                std::process::exit(0);
                            }
                        }
                        _ => bail!("Media playlist not found."),
                    }
                }
                m3u8_rs::Playlist::MediaPlaylist(media) => {
                    println!("{} video stream.", "Downloading".colorize("bold green"));
                    self.progress.current("stream");
                    self.progress.stream = StreamData::new(&self.args.input, &self.args.tempfile());
                    self.progress
                        .json_file(&utils::replace_ext(&self.progress.stream.file, "json"));
                    return Ok(media.segments);
                }
            }
        } else if input_type.is_dash() {
            let mpd = dash::parse(&content)?;
            let master = dash::to_m3u8_as_master(&mpd);

            let uri = if self.args.alternative {
                hls::alternative(&master, self.args.raw_prompts)?
            } else {
                hls::master(&master, &self.args.quality, self.args.raw_prompts)?
            };

            self.progress.current("stream");
            self.progress.stream = StreamData::new(&self.args.input, &self.args.tempfile());
            self.progress
                .json_file(&utils::replace_ext(&self.progress.stream.file, "json"));

            if !self.args.alternative && !self.args.skip {
                self.download_alternative(&master)?;
            }

            let media = dash::to_m3u8_as_media(&mpd, &self.args.input, &uri).unwrap();

            println!(
                "{} {} stream.",
                "Downloading".colorize("bold green"),
                if self.args.alternative {
                    "alternative"
                } else {
                    "video"
                }
            );
            return Ok(media.segments);
        }

        bail!("Only HLS and DASH streams are supported.")
    }

    pub fn download(
        &self,
        segments: &Vec<m3u8_rs::MediaSegment>,
        mut tempfile: String,
    ) -> Result<()> {
        // Check to ensure baseurl is required or not.
        self.args.get_url(&segments[0].uri)?;

        if let Some(output) = &self.args.output {
            if output.ends_with(".ts") {
                tempfile = output.clone();
            }
            println!("File will be saved at {}", tempfile.colorize("cyan"));
        } else {
            println!(
                "Temporary file will be saved at {}",
                tempfile.colorize("cyan")
            );
        }

        let total = segments.len();
        let merger = if self.args.resume {
            Arc::new(Mutex::new(BinarySequence::try_from_json(
                total,
                tempfile.clone(),
                self.progress.json_file.clone(),
            )?))
        } else {
            Arc::new(Mutex::new(BinarySequence::new(
                total,
                tempfile.clone(),
                self.progress.clone(),
            )?))
        };
        merger.lock().unwrap().update()?;

        let pb = Arc::new(Mutex::new(tqdm!(
            total = total,
            unit = "ts".to_owned(),
            dynamic_ncols = true
        )));
        let pool = threadpool::ThreadPool::new(self.args.threads as usize);

        for (i, segment) in segments.iter().enumerate() {
            if self.args.resume {
                let pos = merger.lock().unwrap().position();

                if pos != 0 && pos > i {
                    continue;
                }
            }

            if let Some(m3u8_key) = &segment.key {
                if m3u8_key.method == m3u8_rs::KeyMethod::SampleAES {
                    bail!("SAMPLE-AES encrypted playlists are not supported.")
                }
            }

            let key_url = match &segment.key {
                Some(m3u8_rs::Key {
                    uri: Some(link), ..
                }) => Some(self.args.get_url(link)?),
                _ => None,
            };

            let segment = segment.clone();
            let pb = pb.clone();
            let merger = merger.clone();
            let client = self.client.clone();
            let segment_url = self.args.get_url(&segment.uri)?;
            let total_retries = self.args.retry_count;

            let merger_c = merger.clone();
            let merger_cm = merger_c.lock().unwrap();

            pb.lock().unwrap().set_description(format!(
                "{} / {}",
                utils::format_bytes(merger_cm.stored(), 2).2,
                utils::format_bytes(merger_cm.estimate(), 0).2
            ));
            pb.lock().unwrap().update_to(merger_cm.position());

            pool.execute(move || {
                let mut retries = 0;

                let mut data = loop {
                    let resp = match segment.byte_range {
                        Some(m3u8_rs::ByteRange {
                            length: start,
                            offset: Some(end),
                        }) => client
                            .get(&segment_url)
                            .header(
                                header::RANGE,
                                HeaderValue::from_str(&format!(
                                    "bytes={}-{}",
                                    start,
                                    start + end - 1
                                ))
                                .unwrap(),
                            )
                            .send(),
                        _ => client.get(&segment_url).send(),
                    };

                    // TODO: Check resp errors
                    if let Ok(resp_data) = resp {
                        break resp_data.bytes().unwrap().to_vec();
                    } else if total_retries > retries {
                        pb.lock().unwrap().write(format!(
                            "{} to download segment at index {}.",
                            "RETRYING".colorize("bold yellow"),
                            i
                        ));
                        retries += 1;
                        continue;
                    } else {
                        pb.lock().unwrap().write(format!(
                            "{}: Reached maximum number of retries for segment at index {}.",
                            "Error".colorize("bold red"),
                            i
                        ));
                        std::process::exit(1);
                    }
                };

                // Decrypt
                retries = 0;

                if let Some(eku) = key_url {
                    data = loop {
                        let resp = client.get(&eku).send();

                        if let Ok(resp_data) = resp {
                            let decrypted_data = crate::decrypt::HlsDecrypt::from_key(
                                segment.key.unwrap(),
                                resp_data.bytes().unwrap().to_vec(),
                            )
                            .unwrap()
                            .decrypt(&data);

                            break decrypted_data.unwrap_or_else(|e| {
                                pb.lock().unwrap().write(format!(
                                    "{}: {}",
                                    "Error".colorize("bold red"),
                                    e
                                ));
                                std::process::exit(1);
                            });
                        } else if total_retries > retries {
                            pb.lock().unwrap().write(format!(
                                "{} to download decryption key.",
                                "RETRYING".colorize("bold yellow"),
                            ));
                            retries += 1;
                            continue;
                        } else {
                            pb.lock().unwrap().write(format!(
                                "{}: Reached maximum number of retries to download decryption key.",
                                "Error".colorize("bold red"),
                            ));
                            std::process::exit(1);
                        }
                    };
                }

                let mut merger = merger.lock().unwrap();
                merger.write(i, &data).unwrap();
                merger.flush().unwrap();

                let mut pb = pb.lock().unwrap();
                pb.set_description(format!(
                    "{} / {}",
                    utils::format_bytes(merger.stored(), 2).2,
                    utils::format_bytes(merger.estimate(), 0).2
                ));
                pb.update(1);
            });
        }

        pool.join();
        eprintln!();
        merger.lock().unwrap().flush().unwrap();

        if merger.lock().unwrap().buffered() {
            println!(
                "File {} downloaded successfully.",
                tempfile.colorize("bold green")
            );
        } else {
            bail!(
                "File {} not downloaded successfully.",
                tempfile.colorize("bold red")
            );
        }

        Ok(())
    }

    pub fn transmux_trancode(&mut self) -> Result<()> {
        if let Some(output) = &self.args.output {
            if output.ends_with(".ts") {
                return Ok(());
            }

            let mut args = vec!["-i", &self.progress.stream.file];

            if let Some(audio) = &self.progress.audio {
                args.push("-i");
                args.push(&audio.file);
            }

            if let Some(subtitle) = &self.progress.subtitle {
                args.push("-i");
                args.push(&subtitle.file);
            }

            if std::path::Path::new(output).exists() {
                std::fs::remove_file(output)?;
            }

            if !(output.ends_with(".srt") || output.ends_with(".vtt")) {
                args.push("-c");
                args.push("copy");
            }

            args.push(output);

            println!(
                "Executing {} {}",
                "ffmpeg".colorize("cyan"),
                args.join(" ").colorize("cyan")
            );

            let code = std::process::Command::new("ffmpeg")
                .args(args)
                .stderr(std::process::Stdio::null())
                .spawn()?
                .wait()?;

            if !code.success() {
                bail!("FFMPEG exited with code {}.", code.code().unwrap_or(1))
            }

            if let Some(audio) = &self.progress.audio {
                std::fs::remove_file(&audio.file)?;
            }

            if let Some(subtitle) = &self.progress.subtitle {
                std::fs::remove_file(&subtitle.file)?;
            }

            std::fs::remove_file(&self.progress.stream.file)?;
        }

        if std::path::Path::new(&self.progress.json_file).exists() {
            std::fs::remove_file(&self.progress.json_file)?;
        }
        Ok(())
    }
}
