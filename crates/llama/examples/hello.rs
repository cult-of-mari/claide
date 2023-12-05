use llama::{ClipModel, ModelOptions, SessionOptions};

fn main() {
    llama::init(false);

    let mut options = ModelOptions::new();

    options.set_gpu_layers(33);

    println!("{options:#?}");

    /*let clip_model =
        ClipModel::open("../models/openai_clip-vit-large-patch14-336.gguf", 1).unwrap();

    println!("{clip_model:?}");*/

    let model = options
        .open("../models/teknium_openhermes-2.5-mistral-7b.gguf")
        .unwrap();

    println!("{model:?}");

    let session = SessionOptions::new().with_model(model);

    println!("{session:?}");

    let mut tokens = Vec::new();

    session
        .model()
        .tokenize("hi clyde", &mut tokens, false, false);

    println!("{tokens:?}");

    let mut string = String::new();

    session.model().detokenize(&tokens, &mut string);

    println!("{string:?}");
}
