LLM is OpenHermes 2.5 fine-tune of Mistral 7B, 4-bit quantization, 3992.52 MiB VRAM for 33/33 layers, [download](https://huggingface.co/TheBloke/OpenHermes-2.5-Mistral-7B-GGUF/blob/main/openhermes-2.5-mistral-7b.Q4_K_M.gguf).

CLIP encoder is OpenAI's ViT model, [download](https://huggingface.co/mys/ggml_bakllava-1/resolve/main/mmproj-model-f16.gguf).

Get llama.cpp

```sh
git submodule update --init
```

Run bot (env vars for llama like `RUSTICL_ENABLE` are forwarded)

```sh
cp personality.txt.example personality.txt
CLYDE_TOKEN=xxx cargo run
```
