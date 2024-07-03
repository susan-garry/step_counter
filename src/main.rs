use clap::Parser;
use gfa::parser::GFAParser;
use handlegraph::{
    conversion::from_gfa,
    hashgraph::HashGraph,
    packedgraph::PackedGraph,
    pathhandlegraph::{GraphPathNames, GraphPathsSteps, IntoPathIds},
};
use flatgfa::{FlatGFA, file};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    path: PathBuf,

    #[arg(long)]
    hashgraph: bool,

    #[arg(long)]
    flatgfa: bool,
}

fn hash_performance_check(graph: HashGraph) -> Result<(), Box<dyn std::error::Error>> {
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

fn packed_performance_check(graph: PackedGraph) -> Result<(), Box<dyn std::error::Error>> {
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

fn fgfa_performance_check(graph: FlatGFA) -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    let steps: Vec<(Result<String, _>, usize)> = graph.paths.all().iter()
        .map(|p| {
            (
                graph.get_path_name(p).try_into().unwrap(),
                p.step_count()
            )
        }).collect();
    // for (name, step) in steps {
    // println!("Steps in {}: {}", name?, step);
    // }
    eprintln!("Time: {}ms", now.elapsed().as_millis());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.flatgfa {
        let now = Instant::now();

        eprintln!("Time to parse: {}ms", now.elapsed().as_millis());
        let mmap = file::map_file(cli.path.to_str().unwrap());
        let now = Instant::now();
        let graph = file::view(&mmap);
        eprintln!("Time to build: {}ms", now.elapsed().as_millis());

        fgfa_performance_check(graph)?;

    } else {
        //let path = Path::new("./t.gfa");
        let now = Instant::now();
        let gfa_parser = GFAParser::new();
        let gfa = gfa_parser.parse_file(cli.path)?;
        eprintln!("Time to parse: {}ms", now.elapsed().as_millis());
        if cli.hashgraph {
            let now = Instant::now();
            let graph = from_gfa::<HashGraph, ()>(&gfa);
            eprintln!("Time to build: {}ms", now.elapsed().as_millis());
            hash_performance_check(graph)?;
        } else {
            let now = Instant::now();
            let graph = from_gfa::<PackedGraph, ()>(&gfa);
            eprintln!("Time to build: {}ms", now.elapsed().as_millis());
            packed_performance_check(graph)?;
        }
    }
    Ok(())
}
