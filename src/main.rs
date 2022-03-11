use clap::{ArgEnum, Parser};
use roget::{algorithms, Guesser};

const GAMES: &str = include_str!("../answers.txt");

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, arg_enum)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(ArgEnum, Debug, Clone, Copy)]
enum Implementation {
    Naive,
    Allocs,
    Vecrem,
    Once,
    Precalc,
    Weight,
    Enum,
    Cutoff,
    Popular,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Naive => {
            play::<algorithms::Naive>(args.max);
        }
        Implementation::Allocs => {
            play::<algorithms::Allocs>(args.max);
        }
        Implementation::Vecrem => {
            play::<algorithms::Vecrem>(args.max);
        }
        Implementation::Once => {
            play::<algorithms::OnceInit>(args.max);
        }
        Implementation::Precalc => {
            play::<algorithms::Precalc>(args.max);
        }
        Implementation::Weight => {
            play::<algorithms::Weight>(args.max);
        }
        Implementation::Enum => {
            play::<algorithms::Enumerate>(args.max);
        }
        Implementation::Cutoff => {
            play::<algorithms::Cutoff>(args.max);
        }
        Implementation::Popular => {
            play::<algorithms::Popular>(args.max);
        }
    }
}

fn play<G>(max: Option<usize>)
where
    G: Guesser + Default,
{
    let w = roget::Wordle::new();
    let mut score = 0;
    let mut games = 0;
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = G::default();
        if let Some(s) = w.play(answer, guesser) {
            games += 1;
            score += s;
            println!("guessed '{}' in {}", answer, s);
        } else {
            eprintln!("failed to guess");
        }
    }
    println!("average score: {:.4}", score as f64 / games as f64);
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_10_games_with_cutoff() {
        let w = roget::Wordle::new();
        let results: Vec<_> = crate::GAMES
            .split_whitespace()
            .take(10)
            .filter_map(|answer| w.play(answer, roget::algorithms::Cutoff::new()))
            .collect();

        assert_eq!(results, [4, 4, 4, 4, 4, 5, 4, 5, 4, 2]);
    }
}
