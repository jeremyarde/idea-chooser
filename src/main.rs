use std::{path::PathBuf, process::Command};

use clap::{App, Arg};
use polars::{
    self,
    prelude::{CsvReader, Float32Chunked, IntoSeries, NamedFrom, PolarsError, SerReader, Series},
};

#[derive(Debug, Clone)]
pub enum Message {
    SliderModified((String, f32)),
    ScoresUpdated,
}

#[derive(Debug, Clone)]
struct IdeaItem {
    idea: String,
    fun: f32,
    difficulty: f32,
    market: f32,
    score: f32,
}

fn main() -> Result<(), PolarsError> {
    let matches = App::new("My Super Program")
        .version("1.0")
        // .author("Kevin K. <kbknapp@gmail.com>")
        .about("Calculating which project to work on next.")
        .arg(
            Arg::new("PROJECT")
                // .short('p')
                // .long("project")
                // .value_name("FILE")
                .required(true)
                // .index(0)
                .about("Sets the project file to use"), // .takes_value(true),
        )
        .arg(
            Arg::new("fun")
                .about("Weighting of fun column")
                .required(false)
                .takes_value(true)
                .short('f')
                .long("fun")
                .allow_hyphen_values(true)
                .default_value("1.0"),
        )
        .arg(
            Arg::new("difficulty")
                .about("Weighting of difficulty column")
                .required(false)
                .takes_value(true)
                .short('d')
                .long("difficulty")
                .allow_hyphen_values(true)
                .default_value("1.0"),
        )
        .arg(
            Arg::new("market")
                .about("Weighting of market column")
                .required(false)
                .takes_value(true)
                .short('m')
                .long("market")
                .allow_hyphen_values(true)
                .default_value("1.0"),
        )
        .get_matches();

    // let mut df = CsvReader::from_path("./ideas_2021-09-08.csv")?M
    let mut df = CsvReader::from_path(PathBuf::from(matches.value_of("PROJECT").unwrap()))?
        .infer_schema(None)
        .has_header(true)
        .finish()?;

    // println!("{:?}", df.select_series("Fun Estimate /5"));

    let fun_weight = matches
        .value_of("fun")
        .unwrap()
        .parse::<f32>()
        .unwrap_or(1.0);
    let difficulty_weight = matches
        .value_of("difficulty")
        .unwrap()
        .parse::<f32>()
        .unwrap_or(1.0);
    let market_weight = matches
        .value_of("market")
        .unwrap()
        .parse::<f32>()
        .unwrap_or(1.0);

    let fun_col = "Fun Estimate /5";
    let diff_col = "Difficulty (1-10)";
    let market_col = "Market Potential /5";

    let fun: Vec<f32> = df
        .select_series(fun_col)
        .unwrap()
        .get(0)
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| match x {
            Some(x) => x as f32,
            None => 1.0,
        })
        .collect();

    let diff: Vec<f32> = df
        .select_series(diff_col)
        .unwrap()
        .get(0)
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| match x {
            Some(x) => x as f32,
            None => 1.0,
        })
        .collect();

    let market: Vec<f32> = df
        .select_series(market_col)
        .unwrap()
        .get(0)
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|x| match x {
            Some(x) => x as f32,
            None => 1.0,
        })
        .collect();

    let ideas = df
        .select_series("Idea")
        .unwrap()
        .get(0)
        .unwrap()
        .utf8()
        .unwrap()
        .into_iter()
        .map(|x| match x {
            Some(x) => String::from(x),
            None => String::new(),
        })
        .collect::<Vec<String>>();

    let mut ideaitems: Vec<IdeaItem> = vec![];
    for x in 0..fun.len() {
        let score: f32 =
            fun[x] * fun_weight + diff[x] * difficulty_weight + market[x] * market_weight;

        let mut newidea = IdeaItem {
            idea: ideas[x].clone(),
            fun: fun[x],
            difficulty: diff[x],
            market: market[x],
            score: score,
        };
        // let score = fun * fun_weight;
        // let score: f32 =
        //     fun[x] * fun_weight + diff[x] * difficulty_weight + market[x] * market_weight;
        // scores.push((ideas[x].clone(), score));
        // println!("Idea: {:#?}", scores[x]);
        ideaitems.push(newidea);
    }
    update_scores(&mut ideaitems, fun_weight, difficulty_weight, market_weight);

    ideaitems.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    println!(
        "Settings:\nfun={}, diff={}, market={}",
        fun_weight, difficulty_weight, market_weight
    );
    for item in ideaitems {
        println!("{}: {}", item.score, item.idea);
    }

    Ok(())
}

fn calculate_scores(
    fun: Vec<f32>,
    fun_weight: f32,
    diff: Vec<f32>,
    difficulty_weight: f32,
    market: Vec<f32>,
    market_weight: f32,
    ideas: Vec<String>,
) -> Vec<(String, f32)> {
    let mut scores: Vec<(String, f32)> = vec![];
    for x in 0..fun.len() {
        // let score = fun * fun_weight;
        let score: f32 =
            fun[x] * fun_weight + diff[x] * difficulty_weight + market[x] * market_weight;
        scores.push((ideas[x].clone(), score));
        // println!("Idea: {:#?}", scores[x]);
    }
    scores.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    scores
}

fn update_scores(
    ideas: &mut Vec<IdeaItem>,
    fun_weight: f32,
    difficulty_weight: f32,
    market_weight: f32,
) {
    // let mut scores: Vec<(String, f32)> = vec![];
    for idea in ideas.iter_mut() {
        idea.score = idea.fun * fun_weight
            + idea.market
            + market_weight
            + idea.difficulty * difficulty_weight;
    }
    // ideas
}

fn float_mul(float_val: &Series, by: f32) -> Series {
    float_val
        .i64()
        .unwrap()
        .into_iter()
        .map(|val| val.map(|inner| inner as f32 * by))
        .collect::<Float32Chunked>()
        .into_series()
}

#[test]
fn cli_should_run() {
    let mut command = Command::new("idea-chooser.exe").arg("./ideas_2021-09-08.csv");

    // command.
    // let output = command.output().unwrap();
    // println!("{:?}", output)
}
