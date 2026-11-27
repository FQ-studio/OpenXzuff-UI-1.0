#pragma once
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <string>
#include <string_view>
#include <vector>
#include <utility>
namespace gravity::ai {
struct KnowledgeDocument { std::uint64_t id{}; std::string title; std::string body; };
struct KnowledgeHit { KnowledgeDocument document; double score{}; };
class KnowledgeStore {
public:
 bool execute(std::string_view sql, std::string* error = nullptr);
 bool insert(KnowledgeDocument document, std::string* error = nullptr);
 std::vector<KnowledgeDocument> documents() const;
 bool save(const std::filesystem::path& path, std::string* error = nullptr) const;
 bool load(const std::filesystem::path& path, std::string* error = nullptr);
 std::size_t size() const noexcept;
private:
 bool table_created_{false};
 std::vector<KnowledgeDocument> documents_;
};
class RagEngine {
public:
 explicit RagEngine(const KnowledgeStore& store) : store_(store) {}
 std::vector<KnowledgeHit> search(std::string_view query, std::size_t limit = 5) const;
private: const KnowledgeStore& store_;
};
class QuantizedLinear {
public:
 QuantizedLinear() = default;
 QuantizedLinear(std::size_t input_size, std::size_t output_size, std::vector<std::int8_t> weights, std::vector<float> bias, float weight_scale);
 bool valid() const noexcept;
 std::vector<float> infer(const std::vector<float>& input) const;
private:
 std::size_t input_size_{}, output_size_{};
 std::vector<std::int8_t> weights_;
 std::vector<float> bias_;
 float weight_scale_{};
};
// Native CPU inference primitive. A trained/exported checkpoint is required for useful language generation.
class NativeBrain {
public:
 explicit NativeBrain(QuantizedLinear projection) : projection_(std::move(projection)) {}
 std::vector<float> infer_logits(const std::vector<float>& features) const;
 static std::size_t greedy_token(const std::vector<float>& logits);
private: QuantizedLinear projection_;
};
} // namespace gravity::ai
