#include <mc_schem.h>
#include <stdio.h>

int main(int argc, char **argv) {
  printf("version of mc_schem: %s\n", MC_SCHEM_version_string());

  MC_SCHEM_schematic schem = MC_SCHEM_create_schem();
  MC_SCHEM_destroy_schem(&schem);
  return 0;
}