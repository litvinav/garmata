use std::{collections::HashMap, time::Duration};
use garmata::SendResult;
use clap::{Parser, ValueEnum};

#[derive(Clone, ValueEnum)]
enum Output {
    Stats,
    Csv
}

#[derive(Parser)]
struct CLI {
    /// Location of the configuration file [default: ./configuration.yaml]
    configuration: Option<String>,

    #[arg(short, long, value_enum)]
    /// Output format [default: stats]
    output: Option<Output>,
}

fn main() {
    let cli = CLI::parse();
    let config = cli.configuration.unwrap_or("configuration.yaml".into());
    match garmata::run(config) {
        Ok(results) => match cli.output.unwrap_or(Output::Stats) {
            Output::Csv  => summary_csv(&results),
            Output::Stats => summary_stats(&results)
        },
        Err(e) => eprintln!("{}", e.reason),
    };
}

fn summary_csv(results: &Vec<SendResult>) {
    println!("start timestamp,response status,group,flow,total duration,DNS lookup,connect,TLS handshake,sending,waiting,download");
    for r in results {
        println!(
            "{},{},{},{},{}ms,{}ms,{}ms,{}ms,{}ms,{}ms,{}ms",
            r.start_timestamp,
            r.response_status,
            r.group,
            r.flow,
            r.total_duration.as_millis(),
            r.dns_duration.unwrap_or_default().as_millis(),
            r.connect_duration.as_millis(),
            r.tls_duration.unwrap_or_default().as_millis(),
            r.sending_duration.as_millis(),
            r.waiting_duration.as_millis(),
            r.download_duration.as_millis(),
        );
    }
}

fn summary_stats(results: &Vec<SendResult>) {
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
    for (playlist, map) in formatted {
        println!("Group: {playlist}");
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
                durations
                    .iter()
                    .nth(durations.len() / 2)
                    .unwrap()
                    .as_secs_f32()
            );
            println!(
                "    {:.<68} {}s",
                "p95: ",
                durations
                    .iter()
                    .nth((((durations.len() - 1) as f32) * 0.95).floor() as usize)
                    .unwrap()
                    .as_secs_f32()
            );
            println!(
                "    {:.<68} {}s",
                "max: ",
                durations.last().unwrap().as_secs_f32()
            );
        }
    }
}
