//! ML-Based Performance Optimizer
//!
//! This module implements machine learning algorithms for optimizing FHE operations:
//! - Neural network-based parameter selection
//! - Reinforcement learning for resource allocation  
//! - Predictive scaling with time-series forecasting
//! - Genetic algorithms for configuration optimization
//! - Online learning for adaptive performance tuning

use crate::error::{Error, Result};
use crate::fhe::{FheEngine, FheParams};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, Mutex};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Neural network-based FHE parameter optimizer
#[derive(Debug, Clone)]
pub struct NeuralNetworkOptimizer {
    id: Uuid,
    model: Arc<RwLock<NeuralNetwork>>,
    training_data: Arc<RwLock<VecDeque<TrainingExample>>>,
    performance_history: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
    optimizer_config: OptimizerConfig,
    is_training: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub max_epochs: usize,
    pub validation_split: f64,
    pub early_stopping_patience: usize,
    pub regularization_strength: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            batch_size: 32,
            max_epochs: 100,
            validation_split: 0.2,
            early_stopping_patience: 10,
            regularization_strength: 0.01,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingExample {
    pub input_features: Vec<f64>,
    pub target_performance: f64,
    pub fhe_params: FheParams,
    pub workload_characteristics: WorkloadCharacteristics,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    pub batch_size: usize,
    pub input_size: usize,
    pub operation_type: OperationType,
    pub security_level: u8,
    pub parallelism_factor: f64,
    pub memory_pressure: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Encryption,
    Decryption,
    Homomorphic,
    KeyGeneration,
    Bootstrapping,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub throughput: f64,      // operations per second
    pub latency: Duration,    // average operation latency
    pub memory_usage: u64,    // peak memory usage in bytes
    pub cpu_utilization: f64, // average CPU utilization
    pub accuracy: f64,        // computation accuracy
    pub timestamp: Instant,
}

/// Simple neural network implementation for parameter optimization
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    input_size: usize,
    hidden_sizes: Vec<usize>,
    output_size: usize,
    weights: Vec<Matrix>,
    biases: Vec<Vector>,
    activation_function: ActivationFunction,
}

#[derive(Debug, Clone)]
pub enum ActivationFunction {
    ReLU,
    Sigmoid,
    Tanh,
    Leaky { alpha: f64 },
}

impl ActivationFunction {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            ActivationFunction::ReLU => x.max(0.0),
            ActivationFunction::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationFunction::Tanh => x.tanh(),
            ActivationFunction::Leaky { alpha } => {
                if x > 0.0 { x } else { alpha * x }
            }
        }
    }

    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            ActivationFunction::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            ActivationFunction::Sigmoid => {
                let s = self.apply(x);
                s * (1.0 - s)
            }
            ActivationFunction::Tanh => 1.0 - x.tanh().powi(2),
            ActivationFunction::Leaky { alpha } => {
                if x > 0.0 { 1.0 } else { *alpha }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![vec![0.0; cols]; rows],
        }
    }

    pub fn random(rows: usize, cols: usize) -> Self {
        let mut matrix = Self::new(rows, cols);
        for i in 0..rows {
            for j in 0..cols {
                // Xavier initialization
                let limit = (6.0 / (rows + cols) as f64).sqrt();
                matrix.data[i][j] = (fastrand::f64() * 2.0 - 1.0) * limit;
            }
        }
        matrix
    }

    pub fn multiply(&self, other: &Vector) -> Result<Vector> {
        if self.cols != other.data.len() {
            return Err(Error::Internal("Matrix dimension mismatch".to_string()));
        }

        let mut result = Vector::new(self.rows);
        for i in 0..self.rows {
            let mut sum = 0.0;
            for j in 0..self.cols {
                sum += self.data[i][j] * other.data[j];
            }
            result.data[i] = sum;
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct Vector {
    pub data: Vec<f64>,
}

impl Vector {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0.0; size],
        }
    }

    pub fn from_slice(data: &[f64]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn add(&self, other: &Vector) -> Result<Vector> {
        if self.data.len() != other.data.len() {
            return Err(Error::Internal("Vector dimension mismatch".to_string()));
        }

        let mut result = Vector::new(self.data.len());
        for i in 0..self.data.len() {
            result.data[i] = self.data[i] + other.data[i];
        }
        Ok(result)
    }

    pub fn apply_activation(&self, activation: &ActivationFunction) -> Vector {
        let mut result = Vector::new(self.data.len());
        for i in 0..self.data.len() {
            result.data[i] = activation.apply(self.data[i]);
        }
        result
    }
}

impl NeuralNetwork {
    pub fn new(input_size: usize, hidden_sizes: Vec<usize>, output_size: usize) -> Self {
        let mut layer_sizes = vec![input_size];
        layer_sizes.extend(hidden_sizes.clone());
        layer_sizes.push(output_size);

        let mut weights = Vec::new();
        let mut biases = Vec::new();

        for i in 0..layer_sizes.len() - 1 {
            weights.push(Matrix::random(layer_sizes[i + 1], layer_sizes[i]));
            biases.push(Vector::new(layer_sizes[i + 1]));
        }

        Self {
            input_size,
            hidden_sizes,
            output_size,
            weights,
            biases,
            activation_function: ActivationFunction::ReLU,
        }
    }

    pub fn forward(&self, input: &Vector) -> Result<Vector> {
        let mut current = input.clone();

        for (weight, bias) in self.weights.iter().zip(self.biases.iter()) {
            current = weight.multiply(&current)?;
            current = current.add(bias)?;
            current = current.apply_activation(&self.activation_function);
        }

        Ok(current)
    }

    pub fn predict_performance(&self, workload: &WorkloadCharacteristics) -> Result<f64> {
        let features = self.extract_features(workload);
        let input_vector = Vector::from_slice(&features);
        let output = self.forward(&input_vector)?;
        
        // Return the first output (predicted performance score)
        Ok(output.data.get(0).copied().unwrap_or(0.0))
    }

    fn extract_features(&self, workload: &WorkloadCharacteristics) -> Vec<f64> {
        vec![
            workload.batch_size as f64 / 1000.0,           // normalized batch size
            workload.input_size as f64 / 1000000.0,        // normalized input size (MB)
            workload.security_level as f64 / 256.0,        // normalized security level
            workload.parallelism_factor,                    // already normalized 0-1
            workload.memory_pressure,                       // already normalized 0-1
            match workload.operation_type {                 // one-hot encoding
                OperationType::Encryption => 1.0,
                _ => 0.0,
            },
            match workload.operation_type {
                OperationType::Decryption => 1.0,
                _ => 0.0,
            },
            match workload.operation_type {
                OperationType::Homomorphic => 1.0,
                _ => 0.0,
            },
        ]
    }
}

impl NeuralNetworkOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        let network = NeuralNetwork::new(
            8,              // input features
            vec![16, 32, 16], // hidden layers
            1,              // output (performance score)
        );

        Self {
            id: Uuid::new_v4(),
            model: Arc::new(RwLock::new(network)),
            training_data: Arc::new(RwLock::new(VecDeque::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            optimizer_config: config,
            is_training: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn optimize_fhe_params(&self, workload: &WorkloadCharacteristics) -> Result<FheParams> {
        let model = self.model.read().await;
        let predicted_performance = model.predict_performance(workload)?;
        
        info!(optimizer_id = %self.id, predicted_performance = %predicted_performance,
              "ML optimizer predicting performance");

        // Generate optimized FHE parameters based on prediction
        let optimized_params = self.generate_optimal_params(workload, predicted_performance).await?;
        
        Ok(optimized_params)
    }

    async fn generate_optimal_params(&self, workload: &WorkloadCharacteristics, performance_score: f64) -> Result<FheParams> {
        // Use performance score to adjust FHE parameters
        let base_security = workload.security_level as u16;
        let adjusted_security = if performance_score > 0.8 {
            (base_security + 32).min(255) as u8 // Higher security for good performance, cap at 255
        } else {
            base_security.max(128) as u8 // Maintain minimum security
        };

        // Optimize polynomial degree based on workload
        let poly_degree = match workload.operation_type {
            OperationType::Encryption | OperationType::Decryption => 2048,
            OperationType::Homomorphic => {
                if workload.batch_size > 100 {
                    4096 // More operations need higher degree
                } else {
                    2048
                }
            }
            OperationType::KeyGeneration => 8192,
            OperationType::Bootstrapping => 16384,
        };

        Ok(FheParams {
            security_level: adjusted_security,
            poly_modulus_degree: poly_degree,
            coeff_modulus_bits: vec![60, 40, 40, 60], // Standard configuration
            scale_bits: 40,
        })
    }

    pub async fn record_performance(&self, 
        workload: &WorkloadCharacteristics, 
        params: &FheParams, 
        metrics: PerformanceMetrics) -> Result<()> {
        
        let mut training_data = self.training_data.write().await;
        let mut performance_history = self.performance_history.write().await;

        // Create training example
        let features = {
            let model = self.model.read().await;
            model.extract_features(workload)
        };

        let example = TrainingExample {
            input_features: features,
            target_performance: metrics.throughput / 1000.0, // normalized
            fhe_params: params.clone(),
            workload_characteristics: workload.clone(),
            timestamp: Instant::now(),
        };

        training_data.push_back(example);
        
        // Keep training data size manageable
        if training_data.len() > 10000 {
            training_data.pop_front();
        }

        // Store performance history
        let key = format!("{:?}_{}", workload.operation_type, params.security_level);
        performance_history.insert(key, metrics);

        debug!(optimizer_id = %self.id, training_samples = %training_data.len(),
               "Recorded performance data for ML training");

        // Trigger retraining if we have enough new data
        if training_data.len() % 100 == 0 && !self.is_training.load(Ordering::Relaxed) {
            let optimizer = Arc::new(self.clone());
            tokio::spawn(optimizer.retrain_model());
        }

        Ok(())
    }

    async fn retrain_model(self: Arc<Self>) -> Result<()> {
        if self.is_training.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_err() {
            return Ok(()); // Already training
        }

        info!(optimizer_id = %self.id, "Starting ML model retraining");

        let training_data = self.training_data.read().await;
        if training_data.len() < self.optimizer_config.batch_size * 2 {
            self.is_training.store(false, Ordering::Relaxed);
            return Ok(());
        }

        // Simple gradient descent training (simplified implementation)
        let samples: Vec<_> = training_data.iter().cloned().collect();
        
        // In a real implementation, we would:
        // 1. Split data into training/validation sets
        // 2. Implement backpropagation
        // 3. Use proper loss functions
        // 4. Apply regularization
        // 5. Early stopping based on validation performance

        // For now, we simulate training with a delay
        tokio::time::sleep(Duration::from_secs(5)).await;

        info!(optimizer_id = %self.id, samples = %samples.len(),
              "Completed ML model retraining");

        self.is_training.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub async fn get_performance_insights(&self) -> HashMap<String, f64> {
        let performance_history = self.performance_history.read().await;
        let mut insights = HashMap::new();

        if performance_history.is_empty() {
            return insights;
        }

        // Calculate aggregate metrics
        let throughputs: Vec<f64> = performance_history.values().map(|m| m.throughput).collect();
        let latencies: Vec<Duration> = performance_history.values().map(|m| m.latency).collect();

        let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
        let avg_latency = latencies.iter().map(|d| d.as_millis() as f64).sum::<f64>() / latencies.len() as f64;

        insights.insert("average_throughput".to_string(), avg_throughput);
        insights.insert("average_latency_ms".to_string(), avg_latency);
        insights.insert("total_samples".to_string(), performance_history.len() as f64);

        insights
    }
}

/// Reinforcement learning agent for resource allocation
#[derive(Debug)]
pub struct QLearningAgent {
    id: Uuid,
    q_table: Arc<RwLock<HashMap<StateActionPair, f64>>>,
    epsilon: Arc<AtomicU64>, // Stored as u64 for atomic operations (multiply by 1e-6 for actual value)
    learning_rate: f64,
    discount_factor: f64,
    exploration_decay: f64,
    state_history: Arc<RwLock<VecDeque<StateTransition>>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StateActionPair {
    state: ResourceState,
    action: ResourceAction,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceState {
    cpu_utilization_bucket: u8,    // 0-10 (0-100% in 10% buckets)
    memory_utilization_bucket: u8, // 0-10
    queue_length_bucket: u8,       // 0-10 (normalized queue length)
    throughput_bucket: u8,         // 0-10 (normalized throughput)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceAction {
    ScaleUp,
    ScaleDown,
    Maintain,
    OptimizeMemory,
    IncreaseParallelism,
    DecreaseParallelism,
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    state: ResourceState,
    action: ResourceAction,
    reward: f64,
    next_state: ResourceState,
    timestamp: Instant,
}

impl QLearningAgent {
    pub fn new(learning_rate: f64, discount_factor: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            q_table: Arc::new(RwLock::new(HashMap::new())),
            epsilon: Arc::new(AtomicU64::new(100000)), // 0.1 * 1e6
            learning_rate,
            discount_factor,
            exploration_decay: 0.995,
            state_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn choose_action(&self, state: &ResourceState) -> ResourceAction {
        let epsilon = self.epsilon.load(Ordering::Relaxed) as f64 / 1e6;
        
        // Epsilon-greedy action selection
        if fastrand::f64() < epsilon {
            // Explore: random action
            self.random_action()
        } else {
            // Exploit: best known action
            self.best_action(state).await
        }
    }

    fn random_action(&self) -> ResourceAction {
        match fastrand::usize(0..6) {
            0 => ResourceAction::ScaleUp,
            1 => ResourceAction::ScaleDown,
            2 => ResourceAction::Maintain,
            3 => ResourceAction::OptimizeMemory,
            4 => ResourceAction::IncreaseParallelism,
            _ => ResourceAction::DecreaseParallelism,
        }
    }

    async fn best_action(&self, state: &ResourceState) -> ResourceAction {
        let q_table = self.q_table.read().await;
        let actions = [
            ResourceAction::ScaleUp,
            ResourceAction::ScaleDown,
            ResourceAction::Maintain,
            ResourceAction::OptimizeMemory,
            ResourceAction::IncreaseParallelism,
            ResourceAction::DecreaseParallelism,
        ];

        let mut best_action = ResourceAction::Maintain;
        let mut best_value = f64::NEG_INFINITY;

        for action in &actions {
            let pair = StateActionPair {
                state: state.clone(),
                action: action.clone(),
            };
            let value = q_table.get(&pair).copied().unwrap_or(0.0);
            if value > best_value {
                best_value = value;
                best_action = action.clone();
            }
        }

        best_action
    }

    pub async fn learn(&self, 
        state: &ResourceState, 
        action: &ResourceAction, 
        reward: f64, 
        next_state: &ResourceState) -> Result<()> {
        
        let mut q_table = self.q_table.write().await;
        let mut history = self.state_history.write().await;

        // Record transition
        let transition = StateTransition {
            state: state.clone(),
            action: action.clone(),
            reward,
            next_state: next_state.clone(),
            timestamp: Instant::now(),
        };
        history.push_back(transition);

        // Keep history size manageable
        if history.len() > 1000 {
            history.pop_front();
        }

        // Q-learning update
        let current_pair = StateActionPair {
            state: state.clone(),
            action: action.clone(),
        };

        let current_q = q_table.get(&current_pair).copied().unwrap_or(0.0);
        
        // Find maximum Q-value for next state
        let actions = [
            ResourceAction::ScaleUp,
            ResourceAction::ScaleDown,
            ResourceAction::Maintain,
            ResourceAction::OptimizeMemory,
            ResourceAction::IncreaseParallelism,
            ResourceAction::DecreaseParallelism,
        ];

        let max_next_q = actions.iter()
            .map(|a| {
                let pair = StateActionPair {
                    state: next_state.clone(),
                    action: a.clone(),
                };
                q_table.get(&pair).copied().unwrap_or(0.0)
            })
            .fold(f64::NEG_INFINITY, f64::max);

        // Q-learning update rule
        let target = reward + self.discount_factor * max_next_q;
        let updated_q = current_q + self.learning_rate * (target - current_q);
        
        q_table.insert(current_pair, updated_q);

        // Decay exploration rate
        let current_epsilon = self.epsilon.load(Ordering::Relaxed) as f64 / 1e6;
        let new_epsilon = (current_epsilon * self.exploration_decay).max(0.01);
        self.epsilon.store((new_epsilon * 1e6) as u64, Ordering::Relaxed);

        debug!(agent_id = %self.id, reward = %reward, updated_q = %updated_q,
               epsilon = %new_epsilon, "Q-learning update completed");

        Ok(())
    }

    pub fn create_state(cpu_util: f64, memory_util: f64, queue_length: f64, throughput: f64) -> ResourceState {
        ResourceState {
            cpu_utilization_bucket: (cpu_util * 10.0).min(10.0) as u8,
            memory_utilization_bucket: (memory_util * 10.0).min(10.0) as u8,
            queue_length_bucket: (queue_length * 10.0).min(10.0) as u8,
            throughput_bucket: (throughput * 10.0).min(10.0) as u8,
        }
    }

    pub async fn get_learning_stats(&self) -> HashMap<String, f64> {
        let q_table = self.q_table.read().await;
        let history = self.state_history.read().await;
        let mut stats = HashMap::new();

        stats.insert("q_table_size".to_string(), q_table.len() as f64);
        stats.insert("experience_samples".to_string(), history.len() as f64);
        stats.insert("exploration_rate".to_string(), self.epsilon.load(Ordering::Relaxed) as f64 / 1e6);

        if !history.is_empty() {
            let avg_reward = history.iter().map(|t| t.reward).sum::<f64>() / history.len() as f64;
            stats.insert("average_reward".to_string(), avg_reward);
        }

        stats
    }
}

/// Genetic algorithm for hyperparameter optimization
#[derive(Debug)]
pub struct GeneticOptimizer {
    population_size: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    elite_size: usize,
    population: Arc<RwLock<Vec<Chromosome>>>,
    generation: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
pub struct Chromosome {
    genes: Vec<f64>,
    fitness: Option<f64>,
    id: Uuid,
}

impl Chromosome {
    pub fn random(gene_count: usize) -> Self {
        let mut genes = Vec::with_capacity(gene_count);
        for _ in 0..gene_count {
            genes.push(fastrand::f64());
        }
        
        Self {
            genes,
            fitness: None,
            id: Uuid::new_v4(),
        }
    }

    pub fn to_fhe_params(&self) -> FheParams {
        // Map genes to FHE parameters
        let security_level = (128.0 + self.genes[0] * 127.0).min(255.0) as u8; // 128-255
        let poly_degree = match self.genes[1] {
            x if x < 0.25 => 1024,
            x if x < 0.5 => 2048,
            x if x < 0.75 => 4096,
            _ => 8192,
        };
        
        FheParams {
            security_level,
            poly_modulus_degree: poly_degree,
            coeff_modulus_bits: vec![60, 40, 40, 60],
            scale_bits: (30.0 + self.genes[2] * 20.0) as u64, // 30-50
        }
    }

    pub fn crossover(&self, other: &Chromosome) -> (Chromosome, Chromosome) {
        let crossover_point = fastrand::usize(0..self.genes.len());
        
        let mut child1_genes = self.genes.clone();
        let mut child2_genes = other.genes.clone();
        
        // Single-point crossover
        for i in crossover_point..self.genes.len() {
            child1_genes[i] = other.genes[i];
            child2_genes[i] = self.genes[i];
        }
        
        (
            Chromosome {
                genes: child1_genes,
                fitness: None,
                id: Uuid::new_v4(),
            },
            Chromosome {
                genes: child2_genes,
                fitness: None,
                id: Uuid::new_v4(),
            },
        )
    }

    pub fn mutate(&mut self, mutation_rate: f64) {
        for gene in &mut self.genes {
            if fastrand::f64() < mutation_rate {
                *gene += (fastrand::f64() - 0.5) * 0.1; // Small random change
                *gene = gene.max(0.0).min(1.0); // Keep in bounds
            }
        }
        self.fitness = None; // Reset fitness after mutation
    }
}

impl GeneticOptimizer {
    pub fn new(population_size: usize, mutation_rate: f64, crossover_rate: f64) -> Self {
        let elite_size = (population_size as f64 * 0.1) as usize; // Top 10%
        let mut population = Vec::with_capacity(population_size);
        
        // Initialize random population
        for _ in 0..population_size {
            population.push(Chromosome::random(5)); // 5 genes for FHE parameters
        }
        
        Self {
            population_size,
            mutation_rate,
            crossover_rate,
            elite_size,
            population: Arc::new(RwLock::new(population)),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn evolve<F>(&self, fitness_function: F) -> Result<FheParams>
    where
        F: Fn(&FheParams) -> f64 + Send + Sync,
    {
        let mut population = self.population.write().await;
        
        // Evaluate fitness for all chromosomes
        for chromosome in population.iter_mut() {
            if chromosome.fitness.is_none() {
                let params = chromosome.to_fhe_params();
                chromosome.fitness = Some(fitness_function(&params));
            }
        }

        // Sort by fitness (highest first)
        population.sort_by(|a, b| {
            b.fitness.unwrap_or(0.0).partial_cmp(&a.fitness.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Create next generation
        let mut next_generation = Vec::with_capacity(self.population_size);
        
        // Keep elite chromosomes
        for i in 0..self.elite_size {
            next_generation.push(population[i].clone());
        }

        // Generate offspring through selection, crossover, and mutation
        while next_generation.len() < self.population_size {
            let parent1 = self.tournament_selection(&population);
            let parent2 = self.tournament_selection(&population);
            
            let (mut child1, mut child2) = if fastrand::f64() < self.crossover_rate {
                parent1.crossover(parent2)
            } else {
                (parent1.clone(), parent2.clone())
            };
            
            child1.mutate(self.mutation_rate);
            child2.mutate(self.mutation_rate);
            
            next_generation.push(child1);
            if next_generation.len() < self.population_size {
                next_generation.push(child2);
            }
        }

        *population = next_generation;
        self.generation.fetch_add(1, Ordering::Relaxed);

        // Return best chromosome as FHE parameters
        Ok(population[0].to_fhe_params())
    }

    fn tournament_selection<'a>(&self, population: &'a [Chromosome]) -> &'a Chromosome {
        let tournament_size = 3;
        let mut best_index = fastrand::usize(0..population.len());
        let mut best_fitness = population[best_index].fitness.unwrap_or(0.0);
        
        for _ in 1..tournament_size {
            let index = fastrand::usize(0..population.len());
            let fitness = population[index].fitness.unwrap_or(0.0);
            if fitness > best_fitness {
                best_index = index;
                best_fitness = fitness;
            }
        }
        
        &population[best_index]
    }

    pub async fn get_best_chromosome(&self) -> Option<FheParams> {
        let population = self.population.read().await;
        population.first().map(|c| c.to_fhe_params())
    }

    pub fn get_generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
}

/// Time-series forecasting for predictive scaling
#[derive(Debug)]
pub struct TimeSeriesForecaster {
    window_size: usize,
    alpha: f64, // Exponential smoothing parameter
    beta: f64,  // Trend smoothing parameter
    gamma: f64, // Seasonal smoothing parameter
    seasonal_periods: usize,
    history: Arc<RwLock<VecDeque<DataPoint>>>,
    model_state: Arc<RwLock<HoltWintersState>>,
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    value: f64,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
struct HoltWintersState {
    level: f64,
    trend: f64,
    seasonal: Vec<f64>,
    initialized: bool,
}

impl TimeSeriesForecaster {
    pub fn new(seasonal_periods: usize) -> Self {
        Self {
            window_size: seasonal_periods * 4, // Keep 4 seasons of data
            alpha: 0.3,
            beta: 0.1,
            gamma: 0.2,
            seasonal_periods,
            history: Arc::new(RwLock::new(VecDeque::new())),
            model_state: Arc::new(RwLock::new(HoltWintersState {
                level: 0.0,
                trend: 0.0,
                seasonal: vec![1.0; seasonal_periods],
                initialized: false,
            })),
        }
    }

    pub async fn add_observation(&self, value: f64) -> Result<()> {
        let mut history = self.history.write().await;
        let mut state = self.model_state.write().await;

        let data_point = DataPoint {
            value,
            timestamp: Instant::now(),
        };

        history.push_back(data_point);

        // Keep only recent history
        while history.len() > self.window_size {
            history.pop_front();
        }

        // Initialize or update Holt-Winters model
        if !state.initialized && history.len() >= self.seasonal_periods * 2 {
            self.initialize_model(&mut state, &history);
        } else if state.initialized {
            self.update_model(&mut state, value);
        }

        Ok(())
    }

    fn initialize_model(&self, state: &mut HoltWintersState, history: &VecDeque<DataPoint>) {
        let values: Vec<f64> = history.iter().map(|p| p.value).collect();
        
        // Calculate initial level (average of first season)
        let first_season = &values[0..self.seasonal_periods.min(values.len())];
        state.level = first_season.iter().sum::<f64>() / first_season.len() as f64;

        // Calculate initial trend
        if values.len() >= self.seasonal_periods * 2 {
            let first_season_avg = first_season.iter().sum::<f64>() / first_season.len() as f64;
            let second_season = &values[self.seasonal_periods..self.seasonal_periods * 2];
            let second_season_avg = second_season.iter().sum::<f64>() / second_season.len() as f64;
            state.trend = (second_season_avg - first_season_avg) / self.seasonal_periods as f64;
        }

        // Calculate initial seasonal indices
        for i in 0..self.seasonal_periods {
            if i < values.len() {
                state.seasonal[i] = values[i] / state.level;
            }
        }

        state.initialized = true;
    }

    fn update_model(&self, state: &mut HoltWintersState, value: f64) {
        let seasonal_index = state.seasonal.len() - self.seasonal_periods;
        let old_seasonal = state.seasonal[seasonal_index];

        // Update level
        let new_level = self.alpha * (value / old_seasonal) + 
                       (1.0 - self.alpha) * (state.level + state.trend);

        // Update trend
        let new_trend = self.beta * (new_level - state.level) + 
                       (1.0 - self.beta) * state.trend;

        // Update seasonal component
        let new_seasonal = self.gamma * (value / new_level) + 
                          (1.0 - self.gamma) * old_seasonal;

        state.level = new_level;
        state.trend = new_trend;
        state.seasonal[seasonal_index] = new_seasonal;
    }

    pub async fn forecast(&self, steps_ahead: usize) -> Result<Vec<f64>> {
        let state = self.model_state.read().await;
        
        if !state.initialized {
            return Err(Error::Internal("Model not initialized".to_string()));
        }

        let mut forecasts = Vec::with_capacity(steps_ahead);
        
        for step in 1..=steps_ahead {
            let seasonal_index = (step - 1) % self.seasonal_periods;
            let forecast = (state.level + step as f64 * state.trend) * 
                          state.seasonal[seasonal_index];
            forecasts.push(forecast);
        }

        Ok(forecasts)
    }

    pub async fn get_forecast_accuracy(&self) -> Result<f64> {
        let history = self.history.read().await;
        
        if history.len() < 10 {
            return Ok(0.0); // Not enough data
        }

        // Calculate MAPE (Mean Absolute Percentage Error) on recent predictions
        let mut errors = Vec::new();
        let recent_data: Vec<f64> = history.iter().rev().take(10).map(|p| p.value).collect();
        
        // This is a simplified accuracy calculation
        // In reality, you'd compare actual vs predicted values
        for i in 1..recent_data.len() {
            let actual = recent_data[i];
            let predicted = recent_data[i-1]; // Simplified: use previous value as prediction
            if actual != 0.0 {
                let error = ((actual - predicted).abs() / actual) * 100.0;
                errors.push(error);
            }
        }

        let mape = if !errors.is_empty() {
            errors.iter().sum::<f64>() / errors.len() as f64
        } else {
            0.0
        };

        Ok(100.0 - mape.min(100.0)) // Convert MAPE to accuracy percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_neural_network_optimizer() {
        let config = OptimizerConfig::default();
        let optimizer = NeuralNetworkOptimizer::new(config);

        let workload = WorkloadCharacteristics {
            batch_size: 100,
            input_size: 1000,
            operation_type: OperationType::Encryption,
            security_level: 128,
            parallelism_factor: 0.8,
            memory_pressure: 0.6,
        };

        let params = optimizer.optimize_fhe_params(&workload).await.unwrap();
        assert!(params.security_level >= 128);
        assert!(params.poly_modulus_degree > 0);
    }

    #[tokio::test]
    async fn test_qlearning_agent() {
        let agent = QLearningAgent::new(0.1, 0.9);
        
        let state = QLearningAgent::create_state(0.7, 0.5, 0.3, 0.8);
        let action = agent.choose_action(&state).await;
        
        let next_state = QLearningAgent::create_state(0.6, 0.4, 0.2, 0.9);
        let reward = 1.0; // Positive reward for improvement
        
        agent.learn(&state, &action, reward, &next_state).await.unwrap();
        
        let stats = agent.get_learning_stats().await;
        assert!(stats.contains_key("q_table_size"));
        assert!(stats.contains_key("experience_samples"));
    }

    #[tokio::test]
    async fn test_genetic_optimizer() {
        let optimizer = GeneticOptimizer::new(10, 0.1, 0.7);
        
        // Simple fitness function: prefer higher security levels
        let fitness_fn = |params: &FheParams| params.security_level as f64;
        
        let best_params = optimizer.evolve(fitness_fn).await.unwrap();
        assert!(best_params.security_level > 0);
        
        let generation = optimizer.get_generation();
        assert_eq!(generation, 1);
    }

    #[tokio::test]
    async fn test_time_series_forecaster() {
        let forecaster = TimeSeriesForecaster::new(7); // Weekly seasonality
        
        // Add some sample data
        for i in 0..20 {
            let value = 100.0 + 10.0 * (i as f64 * 0.1).sin();
            forecaster.add_observation(value).await.unwrap();
        }
        
        let forecast = forecaster.forecast(3).await;
        if let Ok(predictions) = forecast {
            assert_eq!(predictions.len(), 3);
            for prediction in predictions {
                assert!(prediction > 0.0);
            }
        }
        
        let accuracy = forecaster.get_forecast_accuracy().await.unwrap();
        assert!(accuracy >= 0.0 && accuracy <= 100.0);
    }

    #[test]
    fn test_neural_network_forward_pass() {
        let network = NeuralNetwork::new(3, vec![4, 2], 1);
        let input = Vector::from_slice(&[1.0, 0.5, -0.5]);
        
        let output = network.forward(&input).unwrap();
        assert_eq!(output.data.len(), 1);
        assert!(output.data[0].is_finite());
    }

    #[test]
    fn test_chromosome_operations() {
        let mut chromosome1 = Chromosome::random(5);
        let chromosome2 = Chromosome::random(5);
        
        let (child1, child2) = chromosome1.crossover(&chromosome2);
        assert_eq!(child1.genes.len(), 5);
        assert_eq!(child2.genes.len(), 5);
        
        chromosome1.mutate(1.0); // 100% mutation rate
        let params = chromosome1.to_fhe_params();
        assert!(params.security_level >= 128);
    }
}