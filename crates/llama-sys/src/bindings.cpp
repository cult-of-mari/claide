#include <clip.h>
#include <ggml-opencl.h>
#include <llama.h>

/// Library
extern "C" void bindings_init(bool numa_aware) {
    llama_backend_init(numa_aware);
}

// CLIP model
extern "C" void *bindings_clip_model_open(const char *path, const int verbosity_level) {
    return static_cast<void *>(clip_model_load(path, verbosity_level));
}

extern "C" void bindings_clip_model_drop(void *model) {
    clip_free(static_cast<clip_ctx *>(model));
}

// CLIP image
extern "C" bool bindings_clip_image_encode(const void *model, const int threads, void *image, float *buf) {
    return clip_image_encode(static_cast<const clip_ctx *>(model), threads, static_cast<struct clip_image_f32 *>(image), buf);
}

extern "C" bool bindings_clip_image_batch_encode(const void *model, const int threads, void *images, float *buf) {
    return clip_image_batch_encode(static_cast<const clip_ctx *>(model), threads, static_cast<struct clip_image_f32_batch *>(images), buf);
}

// Model options
extern "C" void *bindings_model_options_new() {
    return static_cast<void *>(new llama_model_params(llama_model_default_params()));
}

extern "C" uint16_t bindings_model_options_gpu_layers(const void *options) {
    return static_cast<uint16_t>(static_cast<const llama_model_params *>(options)->n_gpu_layers);
}

extern "C" void bindings_model_options_set_gpu_layers(void *options, const uint16_t value) {
    static_cast<llama_model_params *>(options)->n_gpu_layers = static_cast<int32_t>(value);
}

extern "C" bool bindings_model_options_use_mlock(const void *options) {
    return static_cast<const llama_model_params *>(options)->use_mlock;
}

extern "C" void bindings_model_options_set_use_mlock(void *options, const bool value) {
    static_cast<llama_model_params *>(options)->use_mlock = value;
}

extern "C" bool bindings_model_options_use_mmap(const void *options) {
    return static_cast<const llama_model_params *>(options)->use_mmap;
}

extern "C" void bindings_model_options_set_use_mmap(void *options, const bool value) {
    static_cast<llama_model_params *>(options)->use_mmap = value;
}

extern "C" void bindings_model_options_drop(void *options) {
    delete static_cast<llama_model_params *>(options);
}

// Model
extern "C" void *bindings_model_open(const char *path, const void *options) {
    return static_cast<void *>(llama_load_model_from_file(path, *static_cast<const llama_model_params *>(options)));
}

extern "C" void bindings_model_drop(void *model) {
    llama_free_model(static_cast<llama_model *>(model));
}

extern "C" void *bindings_model_new_session(void *model, const void *options) {
    return static_cast<void *>(llama_new_context_with_model(static_cast<llama_model *>(model), *static_cast<const llama_context_params *>(options)));
}

// Session options
extern "C" void *bindings_session_options_new() {
    return static_cast<void *>(new llama_context_params(llama_context_default_params()));
}

extern "C" void bindings_session_options_drop(void *options) {
    delete static_cast<llama_context_params *>(options);
}

// Session
extern "C" void bindings_session_drop(void *session) {
    static_cast<llama_context *>(session);
}
