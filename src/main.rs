use clap::Parser;
use gfa::parser::GFAParser;
use handlegraph::{
    conversion::from_gfa,
    hashgraph::HashGraph,
    packedgraph::PackedGraph,
    pathhandlegraph::{GraphPathNames, GraphPathsSteps, IntoPathIds},
};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: PathBuf,

    #[arg(short, long)]
    hashgraph: bool,
}

fn do_hash_performance_check(graph: HashGraph) -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    let path_ids = graph.path_ids().par_bridge();
    let steps: Vec<(Result<String, _>, usize)> = path_ids
        .map(|id| {
            (
                match graph.get_path_name(id) {
                    Some(name) => String::from_utf8(name.collect::<Vec<_>>()),
                    None => Ok(String::new()),
                },
                match graph.get_path(&id) {
                    Some(path) => path.len(),
                    None => 0,
                },
            )
        })
        .collect();
    // for (name, step) in steps {
    // println!("Steps in {}: {}", name?, step);
    // }
    eprintln!("Time: {}ms", now.elapsed().as_millis());
    Ok(())
}

fn do_packed_performance_check(graph: PackedGraph) -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    let path_ids = graph.path_ids().par_bridge();
    let steps: Vec<(Result<String, _>, usize)> = path_ids
        .map(|id| {
            (
                match graph.get_path_name(id) {
                    Some(name) => String::from_utf8(name.collect::<Vec<_>>()),
                    None => Ok(String::new()),
                },
                match graph.path_steps(id) {
                    Some(steps) => steps.count(),
                    None => 0,
                },
            )
        })
        .collect();
    // for (name, step) in steps {
    // println!("Steps in {}: {}", name?, step);
    // }
    eprintln!("Time: {}ms", now.elapsed().as_millis());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    //let path = Path::new("./t.gfa");
    let now = Instant::now();
    let gfa_parser = GFAParser::new();
    let gfa = gfa_parser.parse_file(cli.path)?;
    eprintln!("Time to parse: {}ms", now.elapsed().as_millis());
    if cli.hashgraph {
        let now = Instant::now();
        let graph = from_gfa::<HashGraph, ()>(&gfa);
        eprintln!("Time to build: {}ms", now.elapsed().as_millis());
        do_hash_performance_check(graph)?;
    } else {
        let now = Instant::now();
        let graph = from_gfa::<PackedGraph, ()>(&gfa);
        eprintln!("Time to build: {}ms", now.elapsed().as_millis());
        do_packed_performance_check(graph)?;
    }
    Ok(())
}
