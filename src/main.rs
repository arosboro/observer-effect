use nokhwa::{Camera, CameraFormat, FrameFormat};
use std::fs;
use std::string::String;
use std::time::{Duration, Instant};

type Trial = fn(u64, String, bool) -> ();

fn candle(trial_length: u64, output_dir: String, active_trial: bool) {
    // set up the Camera
    let mut camera = Camera::new(
        0,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
    )
    .unwrap();
    // open stream
    camera.open_stream().unwrap();

    let start_time = Instant::now();
    let stop_time = start_time + Duration::from_secs(trial_length);
    loop {
        let frame = camera.frame().unwrap();
        let subdir = if active_trial { "trial" } else { "control" };
        let now = Instant::now().elapsed().as_secs();
        let dir = format!("./experiments/{}/{}", output_dir, subdir);
        fs::create_dir_all(&dir).expect("Could not create output directories.");
        let path = format!("{}/{}.jpeg", dir, now);
        frame.save(path).expect("Could not save file.");

        if Instant::now() >= stop_time {
            break;
        }
    }
}

fn rng(trial_length: u64, output_dir: String, active_trial: bool) {
    let mut seed: u8;
    let mut ones: Vec<u8> = Vec::new();
    let mut zeros: Vec<u8> = Vec::new();
    let start_time = Instant::now();
    let stop_time = start_time + Duration::from_secs(trial_length);
    let mut one_count: f64;
    let mut zero_count: f64;
    let mut score: f64;
    loop {
        seed = if rand::random() { 1 } else { 0 };
        if seed == 1 {
            ones.push(seed);
        } else {
            zeros.push(seed);
        }
        println!("{}", seed);
        one_count = ones.len() as f64;
        zero_count = zeros.len() as f64;
        score = capture_score(one_count, zero_count);
        if score > 10.0 {
            bell();
        }
        if Instant::now() >= stop_time {
            break;
        }
    }
    let (total, rem_ones, rem_zeros, exact_ratio, ratio_ones, ratio_zeros, difference, score) =
        record_stats(one_count, zero_count, ones.is_empty(), zeros.is_empty());

    println!("start time: {:?}", start_time);
    println!("stop time: {:?}", stop_time);
    println!("duration: {:?}", trial_length);
    println!("{} total rounds", total);
    println!("{} count of ones", one_count);
    println!("{} count of zeros", zero_count);
    println!("{} difference", difference);
    println!("{:.1}:{:.1} ratio one to zero", rem_ones, rem_zeros);
    println!("{:.6} exact ratio ones by zeros", exact_ratio);
    println!("{:.6}% exact ratio ones by total", ratio_ones * 100.0);
    println!("{:.6}% exact ratio zeros by total", ratio_zeros * 100.0);
    println!("{:.6}% variance", (ratio_ones - ratio_zeros).abs() * 100.0);
    println!("{:.6} score", score);
}

fn capture_score(one_count: f64, zero_count: f64) -> f64 {
    let total: f64 = one_count + zero_count;
    let difference: f64 = (one_count - zero_count).abs();
    difference / total * 1000.0
}

fn record_stats(
    one_count: f64,
    zero_count: f64,
    ones_empty: bool,
    zeros_empty: bool,
) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
    let total: f64 = one_count + zero_count;
    let lcd: f64 = if one_count > zero_count {
        total % zero_count
    } else {
        total % one_count
    };
    let rem_ones: f64 = if lcd != 0.0 { one_count / lcd } else { 1.0 };
    let rem_zeros: f64 = if lcd != 0.0 { zero_count / lcd } else { 1.0 };
    let mut exact_ratio: f64 = 0.0;
    if !(zeros_empty) {
        exact_ratio = one_count / zero_count;
    }
    let mut ratio_ones: f64 = 0.0;
    if !(ones_empty && zeros_empty) {
        ratio_ones = one_count / total;
    }
    let mut ratio_zeros: f64 = 0.0;
    if !(ones_empty && zeros_empty) {
        ratio_zeros = zero_count / total;
    }
    let difference: f64 = (one_count - zero_count).abs();
    let score: f64 = difference / total * 1000.0;

    (
        total,
        rem_ones,
        rem_zeros,
        exact_ratio,
        ratio_ones,
        ratio_zeros,
        difference,
        score,
    )
}

fn sleep(delay: u64) {
    let time_start: Instant = Instant::now();
    let time_stop: Instant = time_start + Duration::from_secs(delay);
    loop {
        if Instant::now() >= time_stop {
            break;
        }
    }
}

fn bell() {
    print!("\x07");
}

fn get_string() -> String {
    let mut string: String = String::new();
    std::io::stdin()
        .read_line(&mut string)
        .expect("Failed to read line");
    string
}

fn get_number() -> u8 {
    let mut input: String = String::new();
    let number: u8;
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let trimmed: &str = input.trim();
    match trimmed.parse::<u8>() {
        Ok(i) => {
            number = i;
        }
        Err(_e) => {
            println!("This was not a unsigned integer: {}", trimmed);
            return 0;
        }
    };
    number
}

fn get_duration() -> u64 {
    let mut input: String = String::new();
    let trimmed: &str;
    let delay: u64;
    println!(" [1]: 30 seconds");
    println!(" [2]: 60 seconds");
    println!(" [3]: 120 seconds");
    println!(" [4]: 5 minutes");
    println!(" [5]: 20 minutes");
    println!(" [6]: 60 minutes");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    trimmed = input.trim();
    match trimmed.parse::<usize>() {
        Ok(i) => {
            if i > 0 && i < 7 {
                let times: [u64; 6] = [30, 60, 120, 5 * 60, 20 * 60, 60 * 60];
                delay = times[i - 1];
            } else {
                delay = 0;
            }
        }
        Err(_e) => {
            println!("This was not a unsigned integer: {}", trimmed);
            return 0;
        }
    };
    delay
}

fn get_experiment() -> Trial {
    let mut input: String = String::new();
    let trimmed: &str;
    let mut experiment: Trial = rng;
    println!(" [1]: RNG");
    println!(" [2]: Candle Light Entropy");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    trimmed = input.trim();
    match trimmed.parse::<usize>() {
        Ok(i) => {
            if i == 1 {
                experiment = rng;
            } else if i == 2 {
                experiment = candle;
            } else {
                println!(
                    "This was not a unsigned integer between 1 and 2: {}",
                    trimmed
                );
            }
        }
        Err(_e) => {
            println!("This was not a unsigned integer: {}", trimmed);
        }
    };
    experiment
}

fn main() {
    // Determine experiment.
    let experiment: Trial = get_experiment();
    // Prompt for delay before starting trial.
    println!("Please enter a delay before starting the trial:");
    let delay: u64 = get_duration();
    // Prompt for length of the trial.
    println!("Please input the desired trial length:");
    let trial_length: u64 = get_duration();
    // Prompt for the number of trials.
    println!("Enter the number of trials to perform:");
    let trials: u8 = get_number();
    // Prompt for a descriptor to classify the trials under.
    println!("Please input a descriptor if you are currently in an altered mental state:");
    let descriptor: String = get_string();
    let now: u64 = Instant::now().elapsed().as_secs();
    bell();
    experiment(delay, format!("experiment-{}", now), false);
    bell();
    for i in 1..=trials {
        println!("Running trial {} of {}", i, trials);
        println!("Altered mental state: {}", descriptor);
        experiment(trial_length, format!("experiment-{:?}", now), true);
    }
}
