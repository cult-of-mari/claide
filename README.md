LLM is OpenHermes 2.5 fine-tune of Mistral 7B, 4-bit quantization, 3992.52 MiB VRAM for 33/33 layers, [download](https://huggingface.co/TheBloke/OpenHermes-2.5-Mistral-7B-GGUF/blob/main/openhermes-2.5-mistral-7b.Q4_K_M.gguf).

CLIP encoder is OpenAI's ViT model, [download](https://huggingface.co/mys/ggml_bakllava-1/resolve/main/mmproj-model-f16.gguf).

Build llama.cpp (git sha1 fbbc42827b2949b95bcde23ce47bb47d006c895d)

```
make LLAMA_CLBLAST=1 -j24
```

Run llama.cpp server:

```bash
./server --model path/to/llm.gguf --n-gpu-layers 33 --mmproj path/to/clip.guuf --ctx-size 32768 --embedding --cont-batching --threads 24
```
