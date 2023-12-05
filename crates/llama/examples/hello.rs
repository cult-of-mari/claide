use llama::{ClipModel, ModelOptions, SessionOptions};

fn main() {
    llama::init(false);

    let mut clip_model =
        ClipModel::open("../models/openai_clip-vit-large-patch14-336.gguf", 1).unwrap();

    let mut model = ModelOptions::new()
        .gpu_layers(33)
        .open("../models/teknium_openhermes-2.5-mistral-7b.gguf")
        .unwrap();

    let _session = SessionOptions::new().with_model(&mut model);
}
