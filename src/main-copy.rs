use blake3::Hasher;
use plotters::prelude::*;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, error};
use env_logger;

const MEMORY_SIZE: usize = 2 * 1024 * 1024; // 2 MiB
const NUM_INSTRUCTIONS: usize = 1_000_000;
const NUM_THREADS: usize = 4;
const TARGET_BLOCK_TIME: Duration = Duration::from_secs(10); // Target block time in seconds

/// Represents a hardware profile that determines mining speed and energy consumption.
#[derive(Debug)]
enum HardwareProfile {
    Fast,
    Medium,
    Slow,
}

impl HardwareProfile {
    fn get_execution_delay(&self) -> Duration {
        match self {
            HardwareProfile::Fast => Duration::from_micros(10),
            HardwareProfile::Medium => Duration::from_micros(50),
            HardwareProfile::Slow => Duration::from_micros(100),
        }
    }

    fn get_energy_per_instruction(&self) -> f64 {
        match self {
            HardwareProfile::Fast => 0.5, // Energy in joules per instruction
            HardwareProfile::Medium => 0.3,
            HardwareProfile::Slow => 0.1,
        }
    }
}

/// Simulates a shared memory area.
struct MemoryArea {
    data: Vec<u8>,
}

impl MemoryArea {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let data = (0..MEMORY_SIZE).map(|_| rng.gen()).collect();
        Self { data }
    }

    fn random_access(&self, index: usize) -> u8 {
        self.data[index % MEMORY_SIZE]
    }
}

/// Represents a mining instruction.
enum Instruction {
    Add(usize, usize),
    Xor(usize, usize),
    ReadMem(usize),
}

impl Instruction {
    fn execute(&self, memory: &MemoryArea, state: &mut u64) {
        match self {
            Instruction::Add(i, j) => *state = state.wrapping_add((*i as u64) + (*j as u64)),
            Instruction::Xor(i, j) => *state ^= (*i as u64) ^ (*j as u64),
            Instruction::ReadMem(i) => *state ^= memory.random_access(*i) as u64,
        }
    }
}

fn generate_random_instructions() -> Vec<Instruction> {
    let mut rng = rand::thread_rng();
    (0..NUM_INSTRUCTIONS)
        .map(|_| match rng.gen_range(0..3) {
            0 => Instruction::Add(rng.gen(), rng.gen()),
            1 => Instruction::Xor(rng.gen(), rng.gen()),
            _ => Instruction::ReadMem(rng.gen()),
        })
        .collect()
}

fn simulate_mining(
    memory: Arc<MemoryArea>,
    instructions: Arc<Vec<Instruction>>,
    thread_id: usize,
    profile: HardwareProfile,
    difficulty_target: u64,
) -> (u64, f64, f64) {
    let mut state: u64 = thread_id as u64;
    let mut energy_consumed: f64 = 0.0;
    let start_time = Instant::now();

    for instruction in instructions.iter() {
        instruction.execute(&memory, &mut state);
        energy_consumed += profile.get_energy_per_instruction();
        thread::sleep(profile.get_execution_delay()); // Simulate hardware speed
    }

    let mut hasher = Hasher::new();
    hasher.update(&state.to_le_bytes());
    let mut result_hash = u64::from_le_bytes(hasher.finalize().as_bytes()[0..8].try_into().unwrap());

    // Simulate network difficulty by repeating until a valid hash is found
    while result_hash > difficulty_target {
        hasher = Hasher::new(); // Reset the hasher
        hasher.update(&state.to_le_bytes());
        state = state.wrapping_add(1); // Safely handle overflow
        result_hash = u64::from_le_bytes(hasher.finalize().as_bytes()[0..8].try_into().unwrap());
        energy_consumed += profile.get_energy_per_instruction();
    }

    let hash_rate = (state as f64) / start_time.elapsed().as_secs_f64(); // Hash rate (hashes per second)
    (result_hash, energy_consumed, hash_rate)
}

fn adjust_difficulty(hash_rates: &[f64], target_block_time: Duration) -> u64 {
    let avg_hash_rate: f64 = hash_rates.iter().sum::<f64>() / hash_rates.len() as f64;
    let target_time_secs = target_block_time.as_secs_f64();
    let new_difficulty = (avg_hash_rate * target_time_secs) as u64;
    new_difficulty.max(1) // Ensure a positive difficulty
}

fn plot_data(data: &[f64], title: &str, x_label: &str, y_label: &str, filename: &str) {
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 40))
        .build_cartesian_2d(
            0..data.len() as u32,
            0.0..data.iter().cloned().fold(0.0, f64::max),
        )
        .unwrap();
    chart.configure_mesh().x_desc(x_label).y_desc(y_label).draw().unwrap();
    chart
        .draw_series(LineSeries::new(
            data.iter().enumerate().map(|(x, &y)| (x as u32, y)),
            &RED,
        ))
        .unwrap();
}

fn main() {
    env_logger::init();
    let memory = Arc::new(MemoryArea::new());
    let instructions = Arc::new(generate_random_instructions());
    let hash_rates = Arc::new(Mutex::new(vec![]));
    let results = Arc::new(Mutex::new(vec![]));

    let profiles = vec![HardwareProfile::Fast, HardwareProfile::Medium, HardwareProfile::Slow];
    let difficulty_target = 0x0000FFFFFFFFFFFF;
    let mut handles = vec![];

    for (i, profile) in profiles.into_iter().enumerate() {
        let memory = Arc::clone(&memory);
        let instructions = Arc::clone(&instructions);
        let hash_rates = Arc::clone(&hash_rates);
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let (hash, energy, hash_rate) =
                simulate_mining(memory, instructions, i, profile, difficulty_target);
            results.lock().unwrap().push((i, hash, energy));
            hash_rates.lock().unwrap().push(hash_rate);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let hash_rate_data = hash_rates.lock().unwrap();
    let energy_data = results.lock().unwrap().iter().map(|(_, _, e)| *e).collect::<Vec<_>>();

    plot_data(
        &energy_data,
        "Energy Consumption",
        "Time (s)",
        "Energy (J)",
        "energy_consumption.png",
    );
    plot_data(
        &hash_rate_data,
        "Hash Rate",
        "Time (s)",
        "Hash Rate (H/s)",
        "hash_rate.png",
    );

    info!("Simulation complete!");
}
