enable_testing()

add_executable(test-nbt_hashmap test-nbt_hashmap.cpp)
target_link_libraries(test-nbt_hashmap PRIVATE mc_schem)
add_test(NAME mc_schem-nbt_hashmap COMMAND test-nbt_hashmap)

add_executable(test-block test-block.cpp)
target_link_libraries(test-block PRIVATE mc_schem)
add_test(NAME mc_schem-block COMMAND test-block)

add_executable(test-region test-region.cpp)
target_link_libraries(test-region PRIVATE mc_schem)
add_test(NAME mc_schem-region COMMAND test-region "${CMAKE_SOURCE_DIR}/test_files")

add_executable(test-entity test-entity.cpp)
target_link_libraries(test-entity PRIVATE mc_schem)
add_test(NAME mc_schem-entity COMMAND test-entity "${CMAKE_SOURCE_DIR}/test_files")

add_executable(test-block_entity test-block_entity.cpp)
target_link_libraries(test-block_entity PRIVATE mc_schem)
add_test(NAME mc_schem-block_entity COMMAND test-block_entity "${CMAKE_SOURCE_DIR}/test_files")

add_executable(test-meta_data_ir test-meta_data_ir.cpp)
target_link_libraries(test-meta_data_ir PRIVATE mc_schem)
add_test(NAME mc_schem-meta_data_ir COMMAND test-meta_data_ir "${CMAKE_SOURCE_DIR}/test_files")