#include "ryml.h"
#include <rust/cxx.h>
#include <memory>

namespace shimmy
{
    inline std::unique_ptr<ryml::Tree> parse(rust::Str text)
    {
        ryml::Tree tree;
        c4::yml::parse_in_arena(c4::csubstr(text.data(), text.size()), &tree);
        return std::make_unique<ryml::Tree>(tree);
    }

    c4::yml::NodeType tree_node_type(const ryml::Tree &tree, size_t node) {
        return tree.type(node);
    }

    void move_node(ryml::Tree &tree, size_t node, size_t after) {
        tree.move(node, after);
    }

    void move_node_to_new_parent(ryml::Tree &tree, size_t node, size_t new_parent, size_t after) {
        tree.move(node, new_parent, after);
    }

    size_t move_node_from_tree(ryml::Tree &tree, ryml::Tree &src, size_t node, size_t new_parent, size_t after) {
        return tree.move(&src, node, new_parent, after);
    }
}
