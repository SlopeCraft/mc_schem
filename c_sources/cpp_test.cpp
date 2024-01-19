#include <mc_schem.h>

int main(int argc, char **argv) {
  static_assert(sizeof(MC_SCHEM_nbt_type) == sizeof(int));
  return 0;
}