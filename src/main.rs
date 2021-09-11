use polars::{
    self,
    prelude::{CsvReader, Float32Chunked, IntoSeries, NamedFrom, PolarsError, SerReader, Series},
};

use iced::slider;
use iced::{Column, Slider, Text};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SliderModified,
}

#[derive(Debug, Clone)]
struct IdeaItem {
    idea: String,
    fun: f32,
    difficulty: f32,
    market: f32,
    score: f32,
}

struct AppState {
    items: Vec<IdeaItem>,
    fun_weight: f32,
    diff_weight: f32,
    market_weight: f32,

    fun_slider: slider::State,
    diff_slider: slider::State,
    market_slider: slider::State,
}

impl AppState {
    pub fn view(&mut self) -> Column<Message> {
        // We use a column: a simple vertical layout
        Column::new()
            .push(
                // The increment button. We tell it to produce an
                // `IncrementPressed` message when pressed
                Slider::new(
                    &mut self.fun_slider,
                    -1.0..=5.0,
                    1.0,
                    update_scores(
                        &self.items,
                        self.fun_weight,
                        self.difficulty_weight,
                        self.market_weight,
                    ),
                )
            )
            .push(
                // We show the value of the counter here
                Text::new(self.value.to_string()).size(50),
            )
            .push(
                // The decrement button. We tell it to produce a
                // `DecrementPressed` message when pressed
                Button::new(&mut self.decrement_button, Text::new("-"))
                    .on_press(Message::DecrementPressed),
            )
    }
}

fn main() -> Result<(), PolarsError> {
    let mut df = CsvReader::from_path("./ideas_2021-09-08.csv")?
        .infer_schema(None)
        .has_header(true)
        .finish()?;

    // println!("{:?}", df.select_series("Fun Estimate /5"));

    let fun_weight = 2.0;
    let difficulty_weight = 0.5;
    let market_weight = 1.0;

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

    let ideaitems: Vec<IdeaItem> = vec![];
    for x in 0..fun.len() {
        let score: f32 =
            fun[x] * fun_weight + diff[x] * difficulty_weight + market[x] * market_weight;

        let mut newidea = IdeaItem {
            idea: ideas[x],
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
    update_scores(&ideaitems, fun_weight, difficulty_weight, market_weight);

    println!("scores:\n{:?}", ideaitems);

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
    mut ideas: &Vec<IdeaItem>,
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
