declare name        "theremotion";
declare version     "1.0";
declare author      "Pierre Lulé";
declare license     "BSD";

import("stdfaust.lib");

// Lead oscillator
lead(res, cutoffNote) = os.sawtooth(f) * v : ve.moog_vcf_2b(res, cutoffFreq)
with {    
    v = hslider("[1]volume", 0.0, 0, 1, 0.001) : si.smoo;

    note = hslider("[0]note", 60, 0, 127, 0.001);
    f = note : ba.midikey2hz : si.smoo;
    cutoffFreq = note + cutoffNote : ba.midikey2hz : si.smoo;
};

leadChord = (res, cutoffNote) <: par(i, 4, vgroup("[3]%i", lead)) :> _ * v
with {
    v = hslider("[0]volume", 0.0, 0, 1, 0.001) : si.smoo;
    cutoffNote = hslider("[1]cutoffNote", 0, -20, 50, 0.001) : si.smoo;
    res = hslider("[2]res", 0, 0, 0.99, 0.001) : si.smoo;
};

feedback(signal)= signal * 0.005;

// Guitar
elecGuitar(stringLength,pluckPosition,mute,gain,trigger) =
    (pm.elecGuitarModel(stringLength,pluckPosition,mute) : co.compressor_mono(20,-10,0,0.1)) ~
    (_  : ef.gate_mono(-20, 0.0001, 0.1, 0.02)) * 0.005 + pm.pluckString(stringLength,1,1,1,gain,trigger);

guitarStrumNote(mute, pitchBend) = elecGuitar(length,0.5,mute,0.5,gate)
    : fi.lowpass(1, f * 2)
with {
    f = note + pitchBend : ba.midikey2hz;
    length = f : pm.f2l;
    gate = button("[0]gate");
    note = hslider("[1]note", 80, 0, 127, 0.001) : si.smoo;
};

guitarStrum(mute, pitchBend) = (mute, pitchBend) <: par(i, 4, vgroup("[3]%i", guitarStrumNote)) :> _;

guitar = guitarStrum(mute, pitchBend)
with {
    mute = hslider("[2]mute", 1, 0.90, 1, 0.001);
    pitchBend = hslider("[3]pitchBend", 0, -1, 1, 0.001) : si.smoo;
};

// Drone
droneNote(detune) = osc(note) + osc(note+detune) + osc(note-detune) : _ * volume
with {
    volume = hslider("[0]volume", 0, 0, 1, 0.001) : si.smoo;
    note = hslider("[1]note", 60, 0, 127, 0.001) : si.smoo;
    osc(note) = os.triangle(ba.midikey2hz(note)) / 5;
};

drone = detune <: par(i, 4, vgroup("[1]%i", droneNote)) :> _
with {
    detune = hslider("[0]detune", 0.1, 0, 0.3, 0.001);
};

echo(s) = s <: ef.echo(10.0, duration, feedback) * mix, s * (1-mix) :> _
with {
    mix = hslider("[0]mix", 1.0, 0, 1, 0.001) : si.smoo;
    duration = hslider("[0]duration[scale:log]", 0.3, 0.01, 3.0, 0.001) : si.smoo;
    feedback = hslider("[1]feedback", 0.3, 0, 1, 0.001);
};

reverb(s) = s <: re.jpverb(t60, damp, size, earlyDiff, modDepth, modFreq, 1, 1, 1, 440, 8000) :> _ <: _ * mix, s * (1-mix) :> _
with {
    mix = hslider("[0]mix", 0.11, 0, 1, 0.001);
    t60 = hslider("[1]time", 3.5, 0.1, 60, 0.001);
    damp = hslider("[2]damp", 0.88, 0, 1, 0.001);
    size = hslider("[3]size", 5.0, 0.5, 5, 0.001);
    earlyDiff = hslider("[4]early_diff", 0.75, 0, 1, 0.001);
    modDepth = hslider("[5]mod_depth", 0.98, 0, 1, 0.001);
    modFreq = hslider("[6]mod_freq", 0.6, 0, 10, 0.001);
};

fx = vgroup("[0]echo", echo) : vgroup("[1]reverb", reverb);

// Mix
process = hgroup("[2]drone", drone) * drone_volume
    + vgroup("[0]lead", leadChord) * lead_volume
    + hgroup("[1]pluck", guitar) * pluck_volume
    : hgroup("[2]fx", fx)
    : _ * master_volume
    <: _, _
with {
    mixGroup(x) = vgroup("[3]mix", x);
    master_volume = mixGroup(hslider("[0]master", 1, 0, 1, 0.001)) : si.smoo;
    drone_volume = mixGroup(hslider("[1]drone", 1, 0, 1, 0.001)) : si.smoo;
    lead_volume = mixGroup(hslider("[2]lead", 1, 0, 1, 0.001)) : si.smoo;
    pluck_volume = mixGroup(hslider("[3]pluck", 1, 0, 1, 0.001)) : si.smoo;
};