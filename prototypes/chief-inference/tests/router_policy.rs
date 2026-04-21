use chief_inference::backends::{BackendId, LocalLlamaCppStub, ModelBackend};
use chief_inference::router::InferenceRouter;
use std::sync::Arc;

#[test]
fn call_override_wins() {
    let mut router = InferenceRouter::new(BackendId("system:default".to_string()));
    let backend = Arc::new(LocalLlamaCppStub::new("llama-7b"));
    let backend_id = backend.id();
    router.register_backend(backend);
    
    let resolved = router.resolve(Some(&backend_id), None, None, None).unwrap();
    assert_eq!(resolved.id().0, "local:llama-cpp-llama-7b");
}
