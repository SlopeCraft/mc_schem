/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.

 This program is distributed in the hope that it will be useful, but WITHOUT ANY
 WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 PARTICULAR PURPOSE.  See the GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#include <array>
#include <cstdint>
#include <filesystem>
#include <mc_schem.hpp>
#include <optional>
#include <string>

#include "utils.hpp"

namespace {

/// Every value the wrapper exposes, so that two meta data objects can be
/// compared field by field in one go
struct meta_snapshot {
  int32_t mc_data_version;
  int64_t time_created;
  int64_t time_modified;
  std::string author;
  std::string name;
  std::string description;
  int32_t litematica_version;
  std::optional<int32_t> litematica_subversion;
  int32_t schem_version;
  pos_t schem_offset;
  std::optional<pos_t> schem_we_offset;
  std::optional<std::string> schem_world_edit_version;
  std::optional<pos_t> schem_origin;
  std::string schem_material;
  std::optional<std::string> schem_editing_platform;

  bool operator==(const meta_snapshot&) const = default;
};

meta_snapshot snapshot_of(const mc_schem::metadata_ir& md) {
  return meta_snapshot{
      .mc_data_version = md.mc_data_version(),
      .time_created = md.time_created(),
      .time_modified = md.time_modified(),
      .author = md.author(),
      .name = md.name(),
      .description = md.description(),
      .litematica_version = md.litematica_version(),
      .litematica_subversion = md.litematica_subversion(),
      .schem_version = md.schem_version(),
      .schem_offset = md.schem_offset(),
      .schem_we_offset = md.schem_we_offset(),
      .schem_world_edit_version = md.schem_world_edit_version(),
      .schem_origin = md.schem_origin(),
      .schem_material = md.schem_material(),
      .schem_editing_platform = md.schem_editing_platform(),
  };
}

}  // namespace

/// Coverage of the meta data IR wrapper. `multi-region01.litematic` provides
/// the meta data of a real file; nothing below depends on the values it happens
/// to carry, apart from the fields that a litematica file always has.
/// C functions behind every section:
///   mc_schem_create_metadata_ir,
///   mc_schem_destroy_metadata_ir (deleter of unique_metadata_ir),
///   mc_schem_clone_metadata_ir,
///   mc_schem_metadata_ir_get_mc_data_version,
///   mc_schem_metadata_ir_get_time_created,
///   mc_schem_metadata_ir_get_time_modified,
///   mc_schem_metadata_ir_get_author, mc_schem_metadata_ir_get_name,
///   mc_schem_metadata_ir_get_description,
///   mc_schem_metadata_ir_get_litematica_version,
///   mc_schem_metadata_ir_get_litematica_subversion,
///   mc_schem_metadata_ir_get_schem_version,
///   mc_schem_metadata_ir_get_schem_offset,
///   mc_schem_metadata_ir_get_schem_we_offset,
///   mc_schem_metadata_ir_get_schem_world_edit_version,
///   mc_schem_metadata_ir_get_schem_editing_platform,
///   mc_schem_metadata_ir_get_schem_origin,
///   mc_schem_metadata_ir_get_schem_material,
///   mc_schem_metadata_ir_set_mc_data_version,
///   mc_schem_metadata_ir_set_time_created,
///   mc_schem_metadata_ir_set_time_modified,
///   mc_schem_metadata_ir_set_author, mc_schem_metadata_ir_set_name,
///   mc_schem_metadata_ir_set_description,
///   mc_schem_metadata_ir_set_litematica_version,
///   mc_schem_metadata_ir_set_litematica_subversion,
///   mc_schem_metadata_ir_set_schem_version,
///   mc_schem_metadata_ir_set_schem_offset,
///   mc_schem_metadata_ir_set_we_offset,
///   mc_schem_metadata_ir_set_schem_world_edit_version,
///   mc_schem_metadata_ir_set_schem_origin,
///   mc_schem_metadata_ir_set_schem_material,
///   mc_schem_metadata_ir_set_schem_editing_platform
int main(int argc, char** argv) {
  using namespace mc_schem;

  // Convention shared with the caller and with ctest: argv[1] is the directory
  // that holds the test fixtures. It is taken as is, the guard below only keeps
  // a missing argument from turning into undefined behaviour.
  MC_SCHEM_CHECK(argc >= 2);
  const std::filesystem::path test_files_dir{argv[1]};

  // mc_schem_create_metadata_ir: whatever data version goes in comes back out
  for (const int32_t data_version : {0, 1, 1343, 3578}) {
    auto created = metadata_ir::create(data_version);
    MC_SCHEM_CHECK(created.has_value());
    MC_SCHEM_CHECK(created.value()->mc_data_version() == data_version);
  }

  // The other fields of a created meta data are the defaults of the Rust side
  // constructor
  {
    auto created = metadata_ir::create(3578);
    MC_SCHEM_CHECK(created.has_value());
    const metadata_ir& md = *created.value();

    // Both timestamps are set to the current time in milliseconds
    MC_SCHEM_CHECK(md.time_created() > 0);
    MC_SCHEM_CHECK(md.time_modified() > 0);
    MC_SCHEM_CHECK(md.author() == "mc_schem");
    MC_SCHEM_CHECK(md.name() == "DefaultMetaDataIR");
    MC_SCHEM_CHECK(md.litematica_version() >= 1);
    MC_SCHEM_CHECK(md.litematica_subversion().has_value());
    MC_SCHEM_CHECK((md.schem_offset() == pos_t{0, 0, 0}));
    MC_SCHEM_CHECK(not md.schem_we_offset().has_value());
    MC_SCHEM_CHECK(not md.schem_world_edit_version().has_value());
    MC_SCHEM_CHECK((md.schem_origin() == pos_t{0, 0, 0}));
    MC_SCHEM_CHECK(md.schem_material() == "Alpha");
    MC_SCHEM_CHECK(not md.schem_editing_platform().has_value());

    // mc_schem_clone_metadata_ir: a copy reports the very same values
    auto copy = md.clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(snapshot_of(*copy) == snapshot_of(md));
  }

  // The meta data that comes with a loaded schematic
  {
    auto schem = load_schematic_or_abort(test_files_dir,
                                         "litematica/multi-region01.litematic");
    const schematic& cs = *schem;
    const metadata_ir* loaded = cs.metadata();
    MC_SCHEM_CHECK(loaded not_eq nullptr);

    // A litematica file always carries its data version and litematica version
    MC_SCHEM_CHECK(loaded->mc_data_version() >= 1);
    MC_SCHEM_CHECK(loaded->litematica_version() >= 1);
    // Both timestamps come from the file, in milliseconds since the epoch
    MC_SCHEM_CHECK(loaded->time_created() > 0);
    MC_SCHEM_CHECK(loaded->time_modified() > 0);
    // The World Edit specific fields only exist for .schem files
    MC_SCHEM_CHECK(not loaded->schem_we_offset().has_value());
    MC_SCHEM_CHECK(not loaded->schem_world_edit_version().has_value());
    MC_SCHEM_CHECK(not loaded->schem_editing_platform().has_value());

    // Cloning carries every field over
    auto copy = loaded->clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(snapshot_of(*copy) == snapshot_of(*loaded));
  }

  // mc_schem_swap_metadata_ir
  {
    auto first = metadata_ir::create(111);
    MC_SCHEM_CHECK(first.has_value());
    auto second = metadata_ir::create(222);
    MC_SCHEM_CHECK(second.has_value());

    const unique_metadata_ir& first_handle = first.value();
    const unique_metadata_ir& second_handle = second.value();
    MC_SCHEM_CHECK(first_handle->mc_data_version() == 111);
    MC_SCHEM_CHECK(second_handle->mc_data_version() == 222);

    first_handle->swap(*second_handle);
    MC_SCHEM_CHECK(first_handle->mc_data_version() == 222);
    MC_SCHEM_CHECK(second_handle->mc_data_version() == 111);
  }

  // The whole setter series. Only that every one of them is called matters
  // here: whether the values come back out again is the business of the Rust
  // side, and asan is what watches the handles on the way through
  {
    auto handle = metadata_ir::create(3578);
    MC_SCHEM_CHECK(handle.has_value());
    metadata_ir& md = *handle.value();

    md.set_mc_data_version(3578);
    md.set_time_created(1707202812934);
    md.set_time_modified(1707202812935);
    md.set_author("unit-test");
    md.set_name("unit-test");
    md.set_description("unit-test");
    md.set_litematica_version(6);
    md.set_litematica_subversion(1);
    md.set_litematica_subversion();
    md.set_schem_version(2);

    const pos_t offset{1, 2, 3};
    md.set_schem_offset(offset);
    const pos_t we_offset{4, 5, 6};
    md.set_schem_we_offset(we_offset);
    md.set_schem_we_offset();

    md.set_schem_world_edit_version("1.13");
    md.set_schem_world_edit_version();

    const pos_t origin{7, 8, 9};
    md.set_schem_origin(origin);
    md.set_schem_origin();

    md.set_schem_material("Alpha");
    md.set_schem_editing_platform("worldedit");
    md.set_schem_editing_platform();

    // After being written to that much the object still clones
    auto copy = md.clone();
    MC_SCHEM_CHECK(copy);
  }

  return 0;
}
