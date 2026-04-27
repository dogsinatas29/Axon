from ollama_runtime import OllamaRuntime
from validator import Validator
from context_builder import ContextBuilder
from pipeline import ExecutionPipeline
from observability import Observability


def main():
    # 1. Runtime
    runtime = OllamaRuntime("http://192.168.0.150:11434")

    print("=== RUNTIME CHECK ===")
    print("PING:", runtime.ping())
    print("MODELS:", runtime.list_models())

    # 2. Config
    config = {
        "agents": {
            "architect": {"model": "mistral:latest"},
            "seniors": [{"model": "mistral:latest"}],
            "juniors": [{"model": "mistral:latest"}]
        },
        "execution": {
            "review_queue_limit": 2,
            "sampling_rate": 0.5
        }
    }

    # 3. Validation
    print("\n=== VALIDATION ===")
    validator = Validator(runtime)
    validation = validator.validate(config)
    print(validation)

    # 4. Context
    print("\n=== CONTEXT ===")
    builder = ContextBuilder()
    context = builder.build(config, validation)
    print(context)

    # 5. Execution
    print("\n=== EXECUTION ===")
    pipeline = ExecutionPipeline(runtime)

    task = "Explain AXON in one sentence"

    # 🔥 raw responses 수집용
    raw_responses = []

    # monkey patch (MVP 방식)
    original_generate = runtime.generate

    def wrapped_generate(model, prompt):
        res = original_generate(model, prompt)
        raw_responses.append(res)
        return res

    runtime.generate = wrapped_generate

    result = pipeline.run(context, task)
    print(result)

    # 6. Observability
    print("\n=== OBSERVABILITY ===")
    obs = Observability()
    report = obs.collect(result, raw_responses)
    print(report)


if __name__ == "__main__":
    main()
