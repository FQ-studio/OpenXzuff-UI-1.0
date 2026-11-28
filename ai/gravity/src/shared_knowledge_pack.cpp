#include "gravity/ai/knowledge.hpp"
#include <array>
namespace gravity::ai {
bool install_shared_knowledge_pack(KnowledgeStore& store, std::string* error) {
 if (!store.execute("CREATE TABLE knowledge (id INTEGER PRIMARY KEY, title TEXT, body TEXT)", error)) return false;
 const std::array<KnowledgeDocument, 10> pack{{
  {1001, "Operating system architecture", "An operating system separates hardware mechanisms from policy. The kernel owns privileged CPU state, memory management, interrupt handling, scheduling, and system-call entry. User applications should use stable ABIs instead of directly accessing devices."},
  {1002, "RISC-V RV64 boot chain", "A typical RV64 platform starts in machine mode firmware such as OpenSBI, which provides the Supervisor Binary Interface. Firmware transfers control to a supervisor-mode kernel. A bootloader such as U-Boot may load the kernel and device tree. The flattened device tree describes CPUs, memory, interrupt controllers, buses, and devices."},
  {1003, "Sv39 virtual memory", "Sv39 is a RISC-V 64-bit virtual-memory scheme with three page-table levels and 4 KiB base pages. Page-table entries carry valid, read, write, execute, user, global, accessed, and dirty permissions. User pages must not expose supervisor memory. Kernel text should be read-execute, read-only data read-only, and writable data non-executable."},
  {1004, "Scheduler and context switching", "A preemptive scheduler selects a runnable task after timer interrupts or blocking events. Context switching preserves the architectural state required to resume a task, including registers, stack pointer, program counter, and address-space context. Timer preemption must acknowledge the timer source and avoid corrupting trap state."},
  {1005, "Process isolation and syscalls", "A system call is an untrusted boundary crossing. Validate syscall numbers, user pointers, buffer lengths, page permissions, handle ownership, and integer overflow before copying data. Copy user memory through bounded routines and never trust a user-provided kernel pointer."},
  {1006, "Virtual file systems and capabilities", "A VFS presents a common file and directory interface over filesystem implementations. A handle should be an unforgeable or generation-checked capability associated with an owning process and explicit rights. Closing a process must invalidate its handles and release resources."},
  {1007, "C++ systems programming", "C++ systems code should make ownership explicit, prefer RAII for resource lifetime, validate sizes before allocation, avoid undefined behavior, and use fixed-width integer types at binary and hardware boundaries. Freestanding kernel code may require restricted standard-library facilities and explicit allocator integration."},
  {1008, "Retrieval augmented generation", "A RAG pipeline ingests attributed documents, splits them into bounded chunks, normalizes text, indexes terms or embeddings, retrieves relevant passages, and supplies those passages as context to a language model. Retrieval scores are not proof of truth. Preserve source, version, and timestamp so answers can be traced and stale material can be replaced."},
  {1009, "Native neural inference", "A neural inference runtime executes a trained model checkpoint. Common operations include embedding lookup, matrix multiplication, normalization, activation functions, attention, softmax, and token sampling. Int8 quantization reduces weight storage and can accelerate matrix operations, but scales and accumulation precision must be handled correctly. An inference engine without trained weights is not a capable language model."},
  {1010, "Knowledge provenance and safe answers", "Knowledge systems should distinguish retrieved facts from model-generated explanations, attach source identifiers, reject malformed or oversized records, and report when evidence is missing. Never fabricate citations, test results, device support, or model capabilities. A failed retrieval should produce an explicit uncertainty response."}
 }};
 for (const auto& document : pack) {
  if (!store.insert(document, error)) return false;
 }
 return true;
}
} // namespace gravity::ai
