#include "ryml/src/inner.rs.h"
// #include "ryml/include/shim.h"

namespace shimmy
{

    struct WriterRust
    {
        rust::Box<RWriter> m_inner;

        WriterRust(rust::Box<RWriter> inner) : m_inner(std::move(inner)) {}

        inline c4::substr _get(bool error_on_excess)
        {
            return m_inner->_get(error_on_excess);
        }

        inline void _do_write(c4::csubstr s)
        {
            m_inner->_do_write(s);
        }

        inline void _do_write(char c)
        {
            m_inner->_do_write_char(c);
        }

        inline void _do_write(c4::yml::RepC rep)
        {
            m_inner->_do_write_repc(rep);
        }

        template <size_t N>
        inline void _do_write(const char (&a)[N])
        {
            rust::Slice<const char> slice(a, N);
            m_inner->_do_write_slice(slice);
        }
    };

    using EmitterRust = c4::yml::Emitter<WriterRust>;

    size_t
    emit_to_rwriter(c4::yml::Tree const &tree, rust::Box<shimmy::RWriter> writer, bool json)
    {
        EmitterRust em(std::move(writer));
        return em.emit(json ? c4::yml::EMIT_JSON : c4::yml::EMIT_YAML, tree, true).len;
    }

}
