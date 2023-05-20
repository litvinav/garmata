use std::{collections::HashMap, time::Duration};

fn main() {
    let config = std::env::args()
        .nth(1)
        .unwrap_or("configuration.yaml".into());
    let result = garmata::run(config);
    // Reformat. Export result here?
    let mut formatted: HashMap<String, HashMap<String, Vec<Duration>>> = HashMap::new();
    for r in result {
        match formatted.get_mut(&r.group) {
            Some(flow_map) => {
                match flow_map.get_mut(&r.flow) {
                    Some(v) => {
                        if let Some(index) = v.iter().position(|&x| x >= r.total_duration) {
                            v.splice(index..index, vec![r.total_duration]);
                        } else {
                            // bigger than any; append
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
    // Display as stdout. Support other formats? 
    for (playlist, map) in formatted {
        println!("Playlist: {playlist}");
        for (flow, durations) in map {
            println!("  Flow: {flow}");
            println!(
                "    {:.<78} {}s",
                "min: ",
                durations.first().unwrap().as_secs_f32()
            );
            println!(
                "    {:.<78} {}s",
                "avg: ",
                durations.iter().map(|d| d.as_secs_f32()).sum::<f32>()
                    / (durations.len() as f32),
            );
            println!(
                "    {:.<78} {}s",
                "p50: ",
                durations
                    .iter()
                    .nth(durations.len() / 2)
                    .unwrap()
                    .as_secs_f32()
            );
            println!(
                "    {:.<78} {}s",
                "p95: ",
                durations
                    .iter()
                    .nth((((durations.len() - 1) as f32) * 0.95).floor() as usize)
                    .unwrap()
                    .as_secs_f32()
            );
            println!(
                "    {:.<78} {}s",
                "max: ",
                durations.last().unwrap().as_secs_f32()
            );
        }
    }
}
