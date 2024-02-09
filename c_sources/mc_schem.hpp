//
// Created by joseph on 2/8/24.
//

#ifndef MC_SCHEM_MC_SCHEM_HPP
#define MC_SCHEM_MC_SCHEM_HPP

#include <mc_schem.h>
#include <memory>
#include <string_view>
#include <span>
#include <type_traits>
#include <utility>
#include <expected>
#include <optional>
#include <string>
#include <cassert>

namespace mc_schem {

  enum class map_key_type : uint8_t {
    string = 0,
    pos_i32 = 1,
  };

  enum class map_value_type : uint8_t {
    string = 0,
    nbt = 1,
    block_entity = 2,
    pending_tick = 3,
  };

  namespace detail {
    std::string_view string_view_schem_to_std(MC_SCHEM_string_view s) noexcept {
      return std::string_view{s.begin, s.end};
    }

    MC_SCHEM_string_view string_view_std_to_schem(std::string_view s) noexcept {
      return MC_SCHEM_string_view{s.data(), s.data() + s.size()};
    }

    template<typename content_t>
    class wrapper {
    protected:
      content_t *handle{nullptr};
    public:
      using handle_type = content_t;

      wrapper() = delete;

      wrapper(content_t *p) : handle{p} {}

      wrapper(const wrapper &) = delete;

      wrapper(wrapper &&src) {
        std::swap(this->handle, src.handle);
      }

      wrapper &operator=(const wrapper &) = delete;

      wrapper &operator=(wrapper &&src) noexcept {
        std::swap(this->handle, src.handle);
      }

      [[nodiscard]] content_t *unwrap_handle() noexcept {
        return this->handle;
      }

      [[nodiscard]] const content_t *unwrap_handle() const noexcept {
        return this->handle;
      }

      void swap(wrapper &another) noexcept {
        std::swap(this->handle, another.handle);
      }

      void reset_handle(content_t *ptr) noexcept {
        this->handle = ptr;
      }
    };


    class deleter {
    public:
      void operator()(MC_SCHEM_block *s) noexcept {
        MC_SCHEM_block_box box{s};
        MC_SCHEM_release_block(&box);
      }
    };

    template<typename content_t, typename c_box_t>
    class box {
    public:
      using handle_t = typename content_t::handle_type;
      static_assert(std::is_same_v<handle_t,
        std::decay_t<decltype(*c_box_t{nullptr}.ptr)>>);
    protected:
      content_t content{nullptr};

      handle_t *handle() noexcept {
        return this->content.unwrap_handle();
      }

      const handle_t *handle() const noexcept {
        return this->content.unwrap_handle();
      }

    public:
      box() = default;

      box(const box &) = delete;

      box(box &&src) {
        this->content.swap(src.content);
      }

      box(c_box_t &&src) : content{src.ptr} {
        src.ptr = nullptr;
      }

      ~box() {
        if (this->handle() != nullptr) {
          deleter{}(this->handle());
        }
      }

      operator bool() const noexcept {
        return this->handle() != nullptr;
      }

      content_t *operator->() noexcept {
        return &this->content;
      }

      const content_t *operator->() const noexcept {
        return &this->content;
      }
    };


    template<map_key_type key_e, typename key_t, map_value_type value_e,
      typename value_t>
    class map_wrapper {
    public:
      using key_ref_type = std::conditional_t<key_e == map_key_type::string, std::string_view, std::span<int, 3>>;

    protected:
      MC_SCHEM_map_ref map_ref;

    public:
      map_wrapper() = delete;

      map_wrapper(MC_SCHEM_map_ref handel) : map_ref{handel} {
        assert(MC_SCHEM_map_get_key_type(&handel) == static_cast<MC_SCHEM_map_key_type>(key_e));
        assert(MC_SCHEM_map_get_value_type(&handel) == static_cast<MC_SCHEM_map_value_type>(value_e));
      }

      map_wrapper(const map_wrapper &&) = delete;

      map_wrapper(map_wrapper &&b) {
        std::swap(this->map_ref, b.map_ref);
      }

      ~map_wrapper() = default;

      static MC_SCHEM_key_wrapper wrap_key(key_ref_type key) noexcept {
        MC_SCHEM_key_wrapper kw;
        if constexpr (key_e == map_key_type::string) {
          kw.string = string_view_schem_to_std(key);
        } else {
          memcpy(reinterpret_cast<void *>( kw.pos), key.data(), key.size());
        }
      }

      static auto unwrap_value(MC_SCHEM_value_wrapper vw) noexcept {
        if constexpr (value_e == map_value_type::string) {
          return vw.string;
        } else if constexpr (value_e == map_value_type::block_entity) {
          return vw.block_entity;
        } else if constexpr (value_e == map_value_type::nbt) {
          return vw.nbt;
        } else {
          return vw.pending_tick;
        }
      }

      [[nodiscard]] size_t size() const noexcept {
        return MC_SCHEM_map_length(&this->map_ref);
      }

      void reserve(size_t new_cap) noexcept {
        MC_SCHEM_map_reserve(&this->map_ref, new_cap);
      }

      [[nodiscard]] bool contains_key(key_ref_type key) noexcept {
        auto k = wrap_key(key);
        return MC_SCHEM_map_contains_key(&this->map_ref,
                                         static_cast<MC_SCHEM_map_key_type>(key_e), &k);
      }

    protected:

      [[nodiscard]] std::optional<value_t> impl_get(key_ref_type key) const noexcept {
        bool ok = false;
        auto k = wrap_key(key);
        auto val_union = MC_SCHEM_map_find(&this->map_ref, key_e, value_e, &k, &ok);
        assert(ok);
        auto val_ptr = unwrap_value(val_union);
        if (val_ptr == nullptr) {
          return std::nullopt;
        }
        return value_t{val_ptr};
      }

    public:

      [[nodiscard]] std::optional<value_t> get(key_ref_type key) noexcept {
        return this->impl_get(key);
      }

      [[nodiscard]] std::optional<const value_t> get(key_ref_type key) const noexcept {
        auto result = this->impl_get(key);
        if (result.has_value()) {
          return std::move(result.value());
        }
        return std::nullopt;
      }
    };
  }

  class block : public detail::wrapper<MC_SCHEM_block> {
  public:
    enum class id_parse_error : uint8_t {
      too_many_colons = 0,
      too_many_left_brackets = 1,
      too_many_right_brackets = 2,
      missing_block_id = 3,
      brackets_not_in_pairs = 4,
      bracket_in_wrong_position = 5,
      colons_in_wrong_position = 6,
      missing_equal_in_attributes = 7,
      too_many_equals_in_attributes = 8,
      missing_attribute_name = 9,
      missing_attribute_value = 10,
      extra_string_after_right_bracket = 11,
      invalid_character = 12,
    };
  public:

    [[nodiscard]] std::string_view get_namespace() const noexcept {
      return detail::string_view_schem_to_std(MC_SCHEM_block_get_namespace(this->handle));
    }

    void set_namespace(std::string_view ns) noexcept {
      MC_SCHEM_block_set_namespace(this->handle, detail::string_view_std_to_schem(ns));
    }

    [[nodiscard]] std::string_view id() const noexcept {
      return detail::string_view_schem_to_std(MC_SCHEM_block_get_id(this->handle));
    }

    void set_id(std::string_view new_id) noexcept {
      MC_SCHEM_block_set_id(this->handle, detail::string_view_std_to_schem(new_id));
    }

    using attribute_map_t = detail::map_wrapper<map_key_type::string, std::string_view, map_value_type::string, std::string_view>;
  protected:
    [[nodiscard]] attribute_map_t impl_attributes() const noexcept {
      auto handle = MC_SCHEM_block_get_attributes(this->handle);
      return attribute_map_t{handle};
    }

  public:
    [[nodiscard]] attribute_map_t attributes() noexcept {
      return this->impl_attributes();
    }

    [[nodiscard]] const attribute_map_t attributes() const noexcept {
      return this->impl_attributes();
    }

    void full_id(std::string &dest) const noexcept {
      dest.resize(256);
      while (true) {
        size_t length = 0;
        MC_SCHEM_block_to_full_id(this->unwrap_handle(), dest.data(), dest.size(), &length);
        if (length != 0) {
          dest.resize(length);
          break;
        }
        dest.resize(dest.size() * 2);
      }
      while (dest.back() == '\0') {
        dest.pop_back();
      }
    }

    std::string full_id() const noexcept {
      std::string result;
      this->full_id(result);
      return result;
    }

    using block_box_t = detail::box<mc_schem::block, MC_SCHEM_block_box>;

    static block_box_t create() noexcept {
      return block_box_t{MC_SCHEM_create_block()};
    }

    static std::expected<block_box_t, id_parse_error> parse_block(std::string_view full_id) noexcept {
      auto result = create();
      MC_SCHEM_block_id_parse_error error;

      const bool ok = MC_SCHEM_parse_block(
        detail::string_view_std_to_schem(full_id),
        result->unwrap_handle(),
        &error);
      if (ok) {
        return std::move(result);
      }
      return std::unexpected(static_cast<id_parse_error>(error));
    }

  };


} // namespace mc_schem


#endif // MC_SCHEM_MC_SCHEM_HPP
