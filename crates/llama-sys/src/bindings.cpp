#include <clip.h>
#include <llama.h>

extern "C" void *bindings_clip_model_open(const char *path, int verbosity_level) {
    return static_cast<void *>(clip_model_load(path, verbosity_level));
}

extern "C" void bindings_init(bool numa_aware) {
    return llama_backend_init(numa_aware);
}

extern "C" void bindings_model_drop(void *model) {
    return llama_free_model(static_cast<llama_model *>(model));
}

extern "C" void *bindings_model_new_session(void *model, const void *options) {
    return static_cast<void *>(llama_new_context_with_model(static_cast<llama_model *>(model), *static_cast<const llama_context_params *>(options)));
}

extern "C" void *bindings_model_open(const char *path, const void *options) {
    return static_cast<void *>(llama_load_model_from_file(path, *static_cast<const llama_model_params *>(options)));
}

extern "C" void *bindings_model_options_new() {
    return static_cast<void *>(new llama_model_params(llama_model_default_params()));
}

extern "C" void bindings_model_options_drop(void *model_options) {
    delete static_cast<llama_model_params *>(model_options);
}
