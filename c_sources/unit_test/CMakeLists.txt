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