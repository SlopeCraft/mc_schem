enable_testing()

add_executable(test-nbt_hashmap test-nbt_hashmap.cpp)
target_link_libraries(test-nbt_hashmap PRIVATE mc_schem)

add_executable(test-block test-block.cpp)
target_link_libraries(test-block PRIVATE mc_schem)