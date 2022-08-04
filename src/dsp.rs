use std::ops::RangeInclusive;
use std::slice;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, StreamConfig};
use faust_state::{DspHandle, Node, StateHandle};
use faust_types::FaustDsp;
use music_note::midi::MidiNote;

use crate::settings::Settings;

fn convert_range(
    value: f32,
    input_range: RangeInclusive<f32>,
    output_range: &RangeInclusive<f32>,
) -> f32 {
    {
        let in_min = *input_range.start();
        let in_max = *input_range.end();
        let out_min = *output_range.start();
        let out_max = *output_range.end();
        ((((value - in_min) * (out_max - out_min)) / (in_max - in_min)) + out_min)
            .clamp(out_min, out_max)
    }
}

/// Smooth step function loosely "sticking" the value to 0 or 1
/// Assumes that value is between 0 and 1
/// https://en.wikipedia.org/wiki/Smoothstep
fn smoothstep(interval: &RangeInclusive<f32>, x: f32) -> f32 {
    let x = (x - interval.start()) / (interval.end() - interval.start());
    x * x * (3.0 - 2.0 * x)
}

pub fn smoothstairs(value: f32, amount: usize, scale: Vec<MidiNote>) -> f32 {
    let scale: Vec<_> = scale
        .windows(2)
        .map(|w| (w[0].into_byte() as f32)..=(w[1].into_byte() as f32))
        .collect();

    if let Some(interval) = scale.iter().find(|interval| interval.contains(&value)) {
        let mut value = value;

        for _ in 0..amount {
            let smooth = smoothstep(interval, value);
            value = interval.start() + smooth * (interval.end() - interval.start());
        }
        return value;
    }
    value
}

#[derive(Debug)]
pub struct Control {
    /// Current value of the control in the DSP
    pub value: f32,

    /// Range declared to the DSP
    pub range: RangeInclusive<f32>,

    /// Name for the DSP
    pub path: String,
}

impl Control {
    pub fn receive(&mut self, state: &mut StateHandle) {
        self.value = *state.get_by_path(&self.path).unwrap();
    }

    pub fn send(&mut self, state: &mut StateHandle) {
        state.set_by_path(&self.path, self.value).unwrap();
    }

    pub fn set_scaled(&mut self, value: f32, value_range: RangeInclusive<f32>) {
        self.value = convert_range(value, value_range, &self.range);
    }
}

impl From<&Node> for Control {
    fn from(node: &Node) -> Self {
        let value = node.init_value();
        let dsp_range = node.min()..=node.max();
        let path = node.path();
        Self {
            value,
            range: dsp_range,
            path,
        }
    }
}

#[derive(Debug)]
pub struct BoolControl {
    /// On-off state
    pub value: bool,

    /// Name for the DSP
    pub path: String,
}

impl BoolControl {
    pub fn receive(&mut self, state: &mut StateHandle) {
        self.value = *state.get_by_path(&self.path).unwrap() > 0.5;
    }

    pub fn send(&mut self, state: &mut StateHandle) {
        state
            .set_by_path(&self.path, if self.value { 1.0 } else { 0.0 })
            .unwrap();
    }
}

impl From<&Node> for BoolControl {
    fn from(node: &Node) -> Self {
        let value = node.init_value() > 0.5;
        let path = node.path();
        Self { value, path }
    }
}

#[derive(Debug)]
pub struct NoteControl {
    /// Current value of the control in the DSP
    pub value: f32,

    /// Name for the DSP
    pub path: String,

    /// Raw note, without autotune
    pub raw_value: f32,

    /// Raw note path
    pub raw_path: String,

    /// Autotune controls
    pub autotune: Control,
}

impl NoteControl {
    pub fn receive(&mut self, state: &mut StateHandle) {
        self.value = *state.get_by_path(&self.path).unwrap();
        self.raw_value = *state.get_by_path(&self.raw_path).unwrap();
        self.autotune.receive(state);
    }

    pub fn send(&mut self, state: &mut StateHandle) {
        state.set_by_path(&self.path, self.value).unwrap();
        state.set_by_path(&self.raw_path, self.raw_value).unwrap();
        self.autotune.send(state);
    }

    pub fn set_scaled(
        &mut self,
        value: f32,
        value_range: RangeInclusive<f32>,
        autotune_value: f32,
        autotune_range: RangeInclusive<f32>,
        settings: &Settings,
    ) {
        let range = settings.note_range_f();
        self.raw_value = convert_range(value, value_range, &range);
        self.autotune.set_scaled(autotune_value, autotune_range);
        self.value = smoothstairs(
            self.raw_value,
            self.autotune.value as usize,
            settings.scale_notes(),
        );
    }
}

impl From<(&Node, &Node, &Node)> for NoteControl {
    fn from(nodes: (&Node, &Node, &Node)) -> Self {
        let value = nodes.0.init_value();
        let path = nodes.0.path();
        let raw_value = nodes.1.init_value();
        let raw_path = nodes.1.path();
        let autotune = nodes.2.into();
        Self {
            value,
            path,
            raw_value,
            raw_path,
            autotune,
        }
    }
}

/// DSP controls
#[derive(Debug)]
pub struct Controls {
    /// Midi note, 0-127
    pub note: NoteControl,
    /// Volume, -96-0
    pub volume: Control,
    /// Filter cutoff, -20-20
    pub cutoff_note: Control,
    /// Filter resonnance, 1-30
    pub resonance: Control,
    /// Supersaw volume
    pub supersaw: Control,
    /// Supersaw detune
    pub detune: Control,
    /// Subosc volume
    pub sub_volume: Control,
    /// Guitar pluck
    pub pluck: BoolControl,
    /// Guitare pluck position
    pub pluck_position: Control,
}

trait NodeByPath {
    fn node_by_path(&self, path: &str) -> Option<&Node>;
}

impl NodeByPath for StateHandle {
    fn node_by_path(&self, path: &str) -> Option<&Node> {
        self.params().values().find(|n| n.path() == path)
    }
}

impl From<&StateHandle> for Controls {
    fn from(state: &StateHandle) -> Self {
        Self {
            note: (
                state.node_by_path("note").unwrap(),
                state.node_by_path("raw_note").unwrap(),
                state.node_by_path("autotune_strength").unwrap(),
            )
                .into(),
            volume: state.node_by_path("volume").unwrap().into(),
            cutoff_note: state.node_by_path("cutoff_note").unwrap().into(),
            resonance: state.node_by_path("res").unwrap().into(),
            supersaw: state.node_by_path("supersaw").unwrap().into(),
            detune: state.node_by_path("detune").unwrap().into(),
            sub_volume: state.node_by_path("sub_volume").unwrap().into(),
            pluck: state.node_by_path("pluck").unwrap().into(),
            pluck_position: state.node_by_path("pluck_position").unwrap().into(),
        }
    }
}

impl Controls {
    /// Read the current control states from the DSP
    pub fn receive(&mut self, state: &mut StateHandle) {
        state.update();
        self.note.receive(state);
        self.volume.receive(state);
        self.cutoff_note.receive(state);
        self.resonance.receive(state);
        self.supersaw.receive(state);
        self.detune.receive(state);
        self.sub_volume.receive(state);
        self.pluck.receive(state);
        self.pluck_position.receive(state);
    }

    pub fn send(&mut self, state: &mut StateHandle) {
        self.note.send(state);
        self.volume.send(state);
        self.cutoff_note.send(state);
        self.resonance.send(state);
        self.supersaw.send(state);
        self.detune.send(state);
        self.sub_volume.send(state);
        self.pluck.send(state);
        self.pluck_position.send(state);
        state.send();
    }
}

/// Run the DSP thread
pub fn run_dsp<T>(mut dsp: DspHandle<T>) -> cpal::Stream
where
    T: FaustDsp<T = f32> + 'static + Send,
{
    // Init cpal client
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();
    // no way of knowing the buffer size in advance?
    let buffer_size: usize = 3000;
    // Get number of inputs and ouputs
    let num_inputs = dsp.num_inputs();
    let num_outputs = dsp.num_outputs();
    // Init DSP with a given sample rate
    let sample_rate = config.sample_rate.0;
    dsp.init(sample_rate as i32);
    // Init output buffers
    let inputs: Vec<Vec<f32>> = vec![vec![0_f32; buffer_size]; num_inputs];
    let mut outputs: Vec<Vec<f32>> = vec![vec![0_f32; buffer_size]; num_outputs];
    // Map our Vec<Vec<f32>> to a Vec<&f[32]> to create a buffer for the faust lib
    let buffer_input: Vec<&[f32]> = inputs
        .iter()
        .map(|input| unsafe { slice::from_raw_parts(input.as_ptr(), buffer_size) })
        .collect();
    // Map our Vec<Vec<f32>> to a Vec<&f[32]> to create a buffer for the faust lib
    let mut buffer_output: Vec<&mut [f32]> = outputs
        .iter_mut()
        .map(|output| unsafe { slice::from_raw_parts_mut(output.as_mut_ptr(), buffer_size) })
        .collect();
    let stream = if let SampleFormat::F32 = sample_format {
        device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let len = data.len();
                    assert!(len <= buffer_size, "Need buffer size of at least {}", len);
                    dsp.update_and_compute(len as i32, &buffer_input[..], &mut buffer_output[..]);

                    for (out, dsp_sample) in data.iter_mut().zip(&outputs[0]) {
                        *out = *dsp_sample;
                        //*sample = Sample::from(&0.0);
                    }
                },
                err_fn,
            )
            .unwrap()
    } else {
        panic!("only looked as f32 rn");
    };
    stream
}
