cmake_minimum_required(VERSION 3.12)
find_package(Corrosion REQUIRED CMAKE_FIND_ROOT_PATH_BOTH)
corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml TARGET gen_invocations)
set_property(
	TARGET gen_invocations
	PROPERTY CORROSION_USE_HOST_BUILD 1
)
