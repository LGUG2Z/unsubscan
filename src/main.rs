#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::redundant_pub_crate)]

use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::ArgEnum;
use clap::Parser;
use color_eyre::Result;
use lazy_static::lazy_static;
use linkify::Link;
use linkify::LinkFinder;
use linkify::LinkKind;
use maud::html;
use regex::Regex;
use serde::Serialize;
use walkdir::DirEntry;
use walkdir::WalkDir;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"From: .*").unwrap();
    static ref SUBJECT_RE: Regex = Regex::new(r"Subject: .*").unwrap();
    static ref LIST_UNSUBSCRIBE_RE: Regex = Regex::new(r"List-Unsubscribe: .*").unwrap();
    static ref URL_FINDER: LinkFinder = {
        let mut url_finder: LinkFinder = LinkFinder::new();
        url_finder.kinds(&[LinkKind::Url]);
        url_finder
    };
    static ref EMAIL_FINDER: LinkFinder = {
        let mut email_finder: LinkFinder = LinkFinder::new();
        email_finder.kinds(&[LinkKind::Email]);
        email_finder
    };
    static ref OUTPUT: PathBuf = {
        let mut temp = temp_dir();
        temp.push("unsubscan.html");
        temp
    };
}

#[derive(Debug, Clone, Serialize)]
struct Unsubscribe {
    from: String,
    list_unsubscribe: String,
    subject: String,
    filename: String,
}

fn process_dent(
    dent: &DirEntry,
    buffer: &mut Vec<u8>,
    unsubscribe_links: &mut Vec<Unsubscribe>,
    debug: bool,
) -> Result<()> {
    if !dent.file_type().is_file() {
        return Ok(());
    }

    if dent.path().extension().map_or(true, |s| s != "eml") {
        return Ok(());
    }

    if debug {
        println!("Scanning email: {}", dent.file_name().to_string_lossy());
    }

    let mut file = File::open(dent.path())?;
    buffer.clear();
    file.read_to_end(buffer)?;

    let content = String::from_utf8_lossy(buffer);

    if let (Some(from_match), Some(list_unsubscribe_match), Some(subject_match)) = (
        FROM_RE.find(&content),
        LIST_UNSUBSCRIBE_RE.find(&content),
        SUBJECT_RE.find(&content),
    ) {
        let from_text = from_match.as_str();
        let list_unsubscribe_text = list_unsubscribe_match.as_str();
        let subject_text = subject_match.as_str();

        let emails = EMAIL_FINDER.links(from_text).collect::<Vec<Link>>();
        let urls = URL_FINDER
            .links(list_unsubscribe_text)
            .collect::<Vec<Link>>();

        if let (Some(email), Some(url)) = (emails.first(), urls.first()) {
            unsubscribe_links.push(Unsubscribe {
                from: email.as_str().to_string(),
                list_unsubscribe: url.as_str().to_string(),
                subject: subject_text
                    .trim_start_matches("Subject: ")
                    .trim()
                    .to_string(),
                filename: dent.file_name().to_string_lossy().to_string(),
            });
        }
    }

    Ok(())
}

fn gen_html(grouped_with_count: &[(&&str, usize, &Vec<&Unsubscribe>)]) -> Result<()> {
    let tpl = html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css";
                title { "unsubscan - Your Unsubscribe Links" }
                script {
                    (maud::PreEscaped(
                        "function toggleVisibility(domain) {
                              var x = document.getElementById(domain);
                              if (x.style.display === 'none') {
                                x.style.display = 'block';
                              } else {
                                x.style.display = 'none';
                              }
                        }"
                    ))
                }
            }

            body {
                section .section {
                    .container {
                        .column.box {
                            .content.has-text-centered {
                                h1 { "unsubscan - Your Unsubscribe Links" }
                                ul {
                                    li { "These are all the unsubscription links that were found from your emails" }
                                    li { "On the left is the list of domains from which unsubscription links were found, sorted by the number of links found" }
                                    li { "Click under each domain to toggle a full list of unsubscribe links from that domain" }
                                    li { "The links are shown with the filename of the email they were found in to help you decide if you want to unsubscribe" }
                                    li { "Generally, more links = more spam = more likely that you'll want to unsubscribe" }
                                    li { "If you have 50+ links from the same domain, you probably don't need to click every single unsubscribe link" }
                                    li { "Remember, you only have to run this against your entire email history once! Subsequent runs can be run against smaller periods (eg. last 3 months)" }
                                }
                            }
                        }
                        #results .columns .is-mobile {
                            .column .is-narrow {
                                @for entry in grouped_with_count {
                                    @let domain = entry.0;
                                    @let count = entry.1;
                                    @let domain_link_id = format!("#{}-toggle", domain);

                                    .box #(domain_link_id) {
                                        p { b { (domain) } }
                                        @let handler = format!("toggleVisibility('{}')", domain);
                                        p { a onclick=(handler) { (count) " emails" } }
                                    }
                                }
                            }
                            .column {
                                @for entry in grouped_with_count {
                                    @let domain = entry.0;
                                    @let unsubs = entry.2;

                                    .box style="display: none" #(domain) {
                                        p { b { (entry.0) } }
                                        @for unsub in unsubs {
                                            p {
                                                a href=(unsub.list_unsubscribe) {
                                                   (unsub.filename)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

        }
    };

    Ok(std::fs::write(&*OUTPUT, tpl.into_string())?)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputFormat {
    Html,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Html
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Directory of EML files to scan for unsubscribe links
    #[clap(value_parser)]
    directory: PathBuf,
    /// The format in which to output scanned unsubscribe links
    #[clap(arg_enum, short, long, default_value_t = OutputFormat::Html)]
    output: OutputFormat,
    /// Enable debug logging
    #[clap(long, action, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut unsubscribe_links = vec![];

    let mut buffer = Vec::new();

    if cli.debug {
        println!(
            "Scanning email files in directory: {}",
            cli.directory.display()
        );
    }

    for dent in WalkDir::new(cli.directory) {
        process_dent(&dent?, &mut buffer, &mut unsubscribe_links, cli.debug)?;
    }

    let mut grouped = HashMap::new();

    for link in &unsubscribe_links {
        let domain: Vec<_> = link.from.split('@').collect();
        grouped.entry(domain[1]).or_insert_with(Vec::new).push(link);
    }

    let mut grouped_count_vec: Vec<_> = grouped.iter().collect();
    let mut grouped_with_count: Vec<_> = grouped_count_vec
        .iter()
        .map(|x| (x.0, x.1.len(), x.1))
        .collect();

    grouped_count_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    grouped_with_count.sort_by(|a, b| b.1.cmp(&a.1));

    match cli.output {
        OutputFormat::Html => {
            gen_html(&grouped_with_count)?;
            open::that(&*OUTPUT)?;
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&grouped_with_count)?);
        }
    };

    Ok(())
}
