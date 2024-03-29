

# Helper macro for LIST_REPLACE
macro(LIST_REPLACE LISTV OLDVALUE NEWVALUE)
    LIST(FIND ${LISTV} ${OLDVALUE} INDEX)
    LIST(INSERT ${LISTV} ${INDEX} ${NEWVALUE})
    MATH(EXPR __INDEX "${INDEX} + 1")
    LIST(REMOVE_AT ${LISTV} ${__INDEX})
endmacro(LIST_REPLACE)

MACRO(install_file_tree LOCATION)
    FOREACH(ifile ${ARGN})
        IF(NOT IS_ABSOLUTE ${ifile})
            GET_FILENAME_COMPONENT(ifile ${ifile} ABSOLUTE
                    BASE_DIR ${CMAKE_SOURCE_DIR})
        ENDIF()
        FILE(RELATIVE_PATH rel ${PUBLIC_INCLUDE_DIRECTORY} ${ifile})
        GET_FILENAME_COMPONENT( dir ${rel} DIRECTORY )
        INSTALL(FILES ${ifile} DESTINATION ${LOCATION}/${dir})
    ENDFOREACH(ifile)
ENDMACRO(install_file_tree)

MACRO(install_platform_library LIBRARY_NAME)
    FOREACH(device ${SUPPORTED_DEVICES})
        INSTALL(TARGETS ${LIBRARY_NAME}-${device} DESTINATION lib EXPORT ${LIBRARY_NAME}-config)
        IF (PUBLIC_INCLUDE_DIRECTORY)
            TARGET_INCLUDE_DIRECTORIES(${LIBRARY_NAME}-${device} PUBLIC
                    ${PLATFORM_PACKAGES_PATH}/include/${LIBRARY_NAME})
        ENDIF (PUBLIC_INCLUDE_DIRECTORY)
    ENDFOREACH(device)
    IF (PUBLIC_INCLUDE_DIRECTORY)
        INSTALL_FILE_TREE(include/${LIBRARY_NAME} ${ARGN})
    ENDIF (PUBLIC_INCLUDE_DIRECTORY)
    INSTALL(EXPORT ${LIBRARY_NAME}-config DESTINATION lib/cmake/${LIBRARY_NAME})
ENDMACRO(install_platform_library)

MACRO(FALLBACK_DEFAULT VAR DEFAULT)
    IF(NOT DEFINED ${VAR})
        SET(${VAR} ${DEFAULT})
    ENDIF()
ENDMACRO()

MACRO(ADD_FILES VAR)
    FILE(RELATIVE_PATH _relPath "${PROJECT_SOURCE_DIR}" "${CMAKE_CURRENT_SOURCE_DIR}")
    FOREACH(_file ${ARGN})
        IF("${VAR}" STREQUAL "SOURCES")
            IF(${USE_ASM_IF_AVAILABLE})
                GET_FILENAME_COMPONENT(_name ${_file} NAME_WE)
                IF(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/${_name}-${TOOLCHAIN_PREFIX}.S")
                    SET(_file ${_name}-${TOOLCHAIN_PREFIX}.S)
                ENDIF()
            ENDIF()
        ENDIF()
        IF(_relPath)
            SET(_file "${_relPath}/${_file}")
        ENDIF()
        SET(${VAR} ${${VAR}} ${_file})
    ENDFOREACH()
    IF(_relPath)
        # propagate SRCS to parent directory
        SET(${VAR} ${${VAR}} PARENT_SCOPE)
    ENDIF()
ENDMACRO()
