use std::{f32::consts::PI, thread};

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use leaprs::*;
use nalgebra::{UnitQuaternion, Vector3};

use crate::{
    conductor_thread::{self, ConductorMessage, HandMessage},
    settings::{Handedness, Settings},
};

const HALF_PI: f32 = PI / 2.0;

/// Start the leap motion thread
pub fn run(
    settings_rx: Receiver<Settings>,
    mut tx: Sender<conductor_thread::ConductorMessage>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut connection =
            Connection::create(ConnectionConfig::default()).expect("Failed to connect");
        connection.open().expect("Failed to open the connection");
        let mut settings = Settings::default();
        loop {
            if let Some(new_settings) = settings_rx.try_iter().last() {
                settings = new_settings;
            }
            if read_and_update(&mut tx, &settings, &mut connection).is_err() {
                // For the lack of better error handling, just assume
                // that the gui thread quit
                return;
            }
        }
    })
}

fn read_and_update(
    tx: &mut Sender<conductor_thread::ConductorMessage>,
    settings: &Settings,
    connection: &mut Connection,
) -> Result<()> {
    match connection.poll(100) {
        Ok(message) => {
            if let Event::Tracking(e) = message.event() {
                let handedness = &settings.system.handedness;

                // List of visible hands
                let hands = e.hands();
                let pitch_hand = hands
                    .iter()
                    .find(|h| h.hand_type() == pitch_hand_type(handedness));
                let volume_hand = hands
                    .iter()
                    .find(|h| h.hand_type() == volume_hand_type(handedness));

                tx.send(ConductorMessage::VisibleHands {
                    left: hands.iter().any(|h| h.hand_type() == HandType::Left),
                    right: hands.iter().any(|h| h.hand_type() == HandType::Right),
                })?;

                if let Some(hand) = pitch_hand {
                    tx.send(ConductorMessage::PitchHand(HandMessage {
                        x_factor: hand.x_factor(),
                        position: hand.position_from_body(),
                        velocity: hand.velocity_from_body(),
                        rotation: hand.rotation_from_body(),
                        pinch: hand.pinch_strength(),
                        grab: hand.grab_strength(),
                    }))?;
                }
                if let Some(hand) = volume_hand {
                    tx.send(ConductorMessage::VolumeHand(HandMessage {
                        x_factor: hand.x_factor(),
                        position: hand.position_from_body(),
                        velocity: hand.velocity_from_body(),
                        rotation: hand.rotation_from_body(),
                        pinch: hand.pinch_strength(),
                        grab: hand.grab_strength(),
                    }))?;
                }
            }
            tx.send(ConductorMessage::LeapError(None))?;
        }
        Err(err) => {
            tx.send(ConductorMessage::LeapError(Some(err.to_string())))?;
        }
    }
    Ok(())
}

fn pitch_hand_type(handedness: &Handedness) -> leaprs::HandType {
    match handedness {
        Handedness::RightHanded => leaprs::HandType::Right,
        Handedness::LeftHanded => leaprs::HandType::Left,
    }
}

fn volume_hand_type(handedness: &Handedness) -> leaprs::HandType {
    match handedness {
        Handedness::RightHanded => leaprs::HandType::Left,
        Handedness::LeftHanded => leaprs::HandType::Right,
    }
}

/// Normalized body direction trait.
/// x direction is from the center of the body to the outside
trait DirectionFromBody {
    /// Factor applied to the x axis to normalize its direction
    fn x_factor(&self) -> f32;
    /// Palm position where the left/right position is normalized:
    /// positive x means arms more open.
    fn position_from_body(&self) -> Vector3<f32>;
    /// Palm velocity where the left/right position is normalized:
    /// positive x means arms more open.
    fn velocity_from_body(&self) -> Vector3<f32>;
    /// Hand twist angle
    fn rotation_from_body(&self) -> Option<f32>;
}

impl DirectionFromBody for Hand<'_> {
    fn x_factor(&self) -> f32 {
        match self.hand_type() {
            // The left hand goes away from the body in the negative x
            HandType::Left => -1.0,
            // The right hand goes away from the body in the positive x
            HandType::Right => 1.0,
        }
    }

    fn position_from_body(&self) -> Vector3<f32> {
        let position = self.palm().position();
        Vector3::new(self.x_factor() * position.x(), position.y(), position.z())
    }

    fn velocity_from_body(&self) -> Vector3<f32> {
        let velocity = self.palm().velocity();
        Vector3::new(self.x_factor() * velocity.x(), velocity.y(), velocity.z())
    }

    fn rotation_from_body(&self) -> Option<f32> {
        let rotation = self.arm().rotation();
        let rotation = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(
            rotation.w(),
            rotation.x(),
            rotation.y(),
            rotation.z(),
        ));
        let angle = -rotation.euler_angles().2 * self.x_factor();
        if angle < PI && angle > -HALF_PI {
            Some(angle)
        } else {
            None
        }
    }
}