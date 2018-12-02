extern crate tensorflow;

use std::num::ParseIntError;
use self::tensorflow::{self as tf};
use phone::{self, Phone};
use fileutil;

pub struct Dnn {
    graph: tf::Graph,
    session: tf::Session,
    label_info: Vec<(usize, usize)>,
    pub spectrum_window_range: (i32, i32),
}

impl Dnn {
    pub fn compute_observation_prob(&mut self, spectrum_window: &[f32], phones: &[Phone]) -> Vec<Vec<f32>> {
        let mut observation_prob = Vec::new();
        for phone in phones.iter() {
            observation_prob.push(vec![0f32; phone.n_states]);
        }

        let output = self.predict(spectrum_window).expect("can't predict dnn output");

        for (index, prob) in output.into_iter().enumerate() {
            let (phone_index, state_num) = self.label_info[index];
            observation_prob[phone_index][state_num] = prob;
        }

        observation_prob
    }

    fn predict(&mut self, input: &[f32]) -> tf::Result<Vec<f32>> {
        let inputs: tf::Tensor<f32> = tf::Tensor::new(&[1u64, input.len() as u64]);
        let inputs = inputs.with_values(input)?;

        let mut args = tf::SessionRunArgs::new();

        let input_op = self.graph.operation_by_name_required("inputs")?;
        let output_op = self.graph.operation_by_name_required("outputs/Softmax")?;

        args.add_feed(&input_op,0, &inputs);
        args.add_target(&output_op);

        let output_token = args.request_fetch(&output_op, 0);

        self.session.run(&mut args)?;

        let outputs: tf::Tensor<f32> = args.fetch(output_token)?;

        Ok(outputs.to_vec())
    }
}

pub fn load(dir: &str, phones: &[Phone]) -> tf::Result<Dnn> {
    println!("tensorflow version: {:?}", tensorflow::version()?);

    let mut graph = tf::Graph::new();

    let session = tf::Session::from_saved_model(
        &tf::SessionOptions::new(),
        &["serve"],
        &mut graph,
        dir,
    )?;

    let label_info_path = format!("{}/label_info.txt", dir);
    let label_info = load_label_info(&label_info_path, phones).unwrap();

    let range_path = format!("{}/spectrum_window_range.txt", dir);
    let spectrum_window_range = load_spectrum_window_range(&range_path).unwrap();

    println!("graph operations");
    for op in graph.operation_iter() {
        println!("{:?}", op.name()?);
    }

    Ok(Dnn { graph, session, label_info, spectrum_window_range })
}

fn load_label_info(path: &str, phones: &[Phone]) -> Result<Vec<(usize, usize)>, ParseIntError> {
    let lines = fileutil::read_lines(std::ffi::OsStr::new(path));

    let mut label_info = vec![(0 as usize, 0 as usize); lines.len()];
    for line in lines.iter() {
        let elements: Vec<_> = line.split_whitespace().collect();
        let label_index: usize = elements[0].parse()?;
        let phone_name = elements[1];
        let state_num: usize = elements[2].parse()?;

        let phone_index = phone::find(phone_name, phones).index;
        label_info[label_index] = (phone_index, state_num - 1);
    }

    Ok(label_info)
}

fn load_spectrum_window_range(path: &str) -> Result<(i32, i32), ParseIntError> {
    let lines = fileutil::read_lines(std::ffi::OsStr::new(path));

    let elements: Vec<_> = lines[0].split_whitespace().collect();

    Ok((elements[0].parse()?, elements[1].parse()?))
}
