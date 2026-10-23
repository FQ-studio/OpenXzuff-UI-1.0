#include "openxzuff/kernel/contracts.hpp"
namespace openxzuff::kernel {
// Host-test marker only: hardware boot and trap entry are not implemented.
std::uint64_t contract_version() noexcept { return 1; }
} // namespace openxzuff::kernel
