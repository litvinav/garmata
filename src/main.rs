// MIT License - free as in freedom; Full license in the LICENSE file
use async_std::task::block_on;
use clap::{Parser, ValueEnum};
use garmata::http::HttpResult;
use std::{collections::HashMap, time::Duration};

#[derive(Clone, ValueEnum, PartialEq, Eq)]
enum Output {
    Stats,
    Csv,
    Debug,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Location of the test configuration file [default: ./configuration.yaml]
    configuration: Option<String>,

    #[arg(short, long, value_enum)]
    /// Output format [default: stats]
    output: Option<Output>,
}

fn main() {
    let cli = Cli::parse();
    let config = cli.configuration.unwrap_or("configuration.yaml".into());
    let is_debug = cli.output == Some(Output::Debug);
    match block_on(garmata::run(config, is_debug)) {
        Ok(results) => match cli.output.unwrap_or(Output::Stats) {
            Output::Csv => summary_csv(&results),
            Output::Stats => summary_stats(&results),
            Output::Debug => {},
        },
        Err(e) => eprintln!("{}", e.reason),
    };
}

fn summary_csv(results: &Vec<HttpResult>) {
    println!("start timestamp,response status,group,flow,total in μs,DNS lookup in μs,Connection in μs,TLS handshake in μs,redirecting in μs,sending in μs,waiting in μs,downloading in μs");
    for r in results {
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            r.start_timestamp,
            r.response_status,
            r.group,
            r.flow,
            r.total_duration.as_micros(),
            r.dns_duration.as_micros(),
            r.connect_duration.as_micros(),
            r.tls_duration.as_micros(),
            r.redirect_duration.as_micros(),
            r.sending_duration.as_micros(),
            r.waiting_duration.as_micros(),
            r.download_duration.as_micros(),
        );
    }
}

fn summary_stats(results: &Vec<HttpResult>) {
    let mut formatted: HashMap<String, HashMap<String, Vec<Duration>>> = HashMap::new();
    for r in results {
        match formatted.get_mut(&r.group) {
            Some(flow_map) => {
                match flow_map.get_mut(&r.flow) {
                    Some(v) => {
                        if let Some(index) = v.iter().position(|&x| x >= r.total_duration) {
                            v.splice(index..index, vec![r.total_duration]);
                        } else {
                            v.push(r.total_duration);
                        }
                    }
                    None => {
                        flow_map.insert(r.flow.clone(), vec![r.total_duration]);
                    }
                };
            }
            None => {
                let mut map = HashMap::new();
                map.insert(r.flow.clone(), vec![r.total_duration]);
                formatted.insert(r.group.clone(), map);
            }
        };
    }
    for (group, map) in &formatted {
        let total = &map.iter().map(|(_, timings)| timings.len()).sum::<usize>();
        let pad = if group.is_empty() { "" } else { " " };
        println!("Group: {group}{pad}({total} requests total)");
        for (flow, durations) in map {
            println!("  Flow: {flow}");
            println!(
                "    {:.<68} {}s",
                "min: ",
                durations.first().unwrap().as_secs_f32()
            );
            println!(
                "    {:.<68} {}s",
                "avg: ",
                durations.iter().map(|d| d.as_secs_f32()).sum::<f32>() / (durations.len() as f32),
            );
            println!(
                "    {:.<68} {}s",
                "p50: ",
                durations[durations.len() / 2].as_secs_f32()
            );
            println!(
                "    {:.<68} {}s",
                "p95: ",
                durations[((durations.len() as f32) * 0.95).floor() as usize].as_secs_f32()
            );
            println!(
                "    {:.<68} {}s",
                "max: ",
                durations.last().unwrap().as_secs_f32()
            );
        }
    }
}
