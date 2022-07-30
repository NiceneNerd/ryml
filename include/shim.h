#include "ryml.h"
#include <rust/cxx.h>
#include <mutex>
#include <memory>
#pragma once
namespace shimmy
{

    struct RWriter;

    size_t emit_to_rwriter(c4::yml::Tree const &tree, rust::Box<RWriter> writer, bool json);

    class RymlError : public std::runtime_error
    {
    public:
        using std::runtime_error::runtime_error;
    };

    void init_ryml_once()
    {
        static std::once_flag s_flag;
        std::call_once(s_flag, []
                       {
    ryml::Callbacks callbacks = ryml::get_callbacks();
    callbacks.m_error = [](const char* msg, size_t msg_len, ryml::Location loc, void*) {
      throw RymlError(std::string(msg, msg_len) + "\n    at " + std::string(loc.name.data(), loc.name.len) + ":" + std::to_string(loc.line));
    };
    ryml::set_callbacks(callbacks);
    c4::set_error_callback([](const char* msg, size_t msg_size) {
      throw RymlError("RymlError (c4): " + std::string(msg, msg_size));
    }); });
    }

    inline std::unique_ptr<ryml::Tree> new_tree()
    {
        init_ryml_once();
        return std::make_unique<ryml::Tree>();
    }

    inline std::unique_ptr<ryml::Tree> clone_tree(const ryml::Tree &tree)
    {
        init_ryml_once();
        return std::make_unique<ryml::Tree>(tree);
    }

    inline std::unique_ptr<ryml::Tree> parse(rust::Str text)
    {
        init_ryml_once();
        ryml::Tree tree = c4::yml::parse_in_arena(c4::csubstr(text.data(), text.size()));
        return std::make_unique<ryml::Tree>(std::move(tree));
    }

    inline std::unique_ptr<ryml::Tree> parse_in_place(char *text, size_t len)
    {
        init_ryml_once();
        ryml::Tree tree = c4::yml::parse_in_place(c4::substr(text, len));
        return std::make_unique<ryml::Tree>(std::move(tree));
    }

    inline c4::yml::NodeType tree_node_type(const ryml::Tree &tree, size_t node)
    {
        return tree.type(node);
    }

    inline void move_node(ryml::Tree &tree, size_t node, size_t after)
    {
        tree.move(node, after);
    }

    inline void move_node_to_new_parent(ryml::Tree &tree, size_t node, size_t new_parent, size_t after)
    {
        tree.move(node, new_parent, after);
    }

    inline size_t move_node_from_tree(ryml::Tree &tree, ryml::Tree &src, size_t node, size_t new_parent, size_t after)
    {
        return tree.move(&src, node, new_parent, after);
    }
}
