/*
mc_schem is a rust library to generate, load, manipulate and save minecraft
schematic files. Copyright (C) 2024  joseph

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

#ifndef MC_SCHEM_MC_SCHEM_HPP
#define MC_SCHEM_MC_SCHEM_HPP

#include <mc_schem.h>

#include <array>
#include <cassert>
#include <exception>
#include <expected>
#include <format>
#include <functional>
#include <istream>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <variant>

namespace mc_schem {

  [[nodiscard]] std::string_view lib_version_string() noexcept {
    return MC_SCHEM_version_string();
  }

  struct version {
    uint16_t major;
    uint16_t minor;
    uint16_t patch;
  };
  [[nodiscard]] version lib_version() noexcept {
    return version{MC_SCHEM_version_major(), MC_SCHEM_version_minor(),
                   MC_SCHEM_version_patch()};
  }

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
    [[nodiscard]] std::string_view string_view_schem_to_std(
      MC_SCHEM_string_view s) noexcept {
      return std::string_view{s.begin, s.end};
    }

    [[nodiscard]] MC_SCHEM_string_view string_view_std_to_schem(
      std::string_view s) noexcept {
      return MC_SCHEM_string_view{s.data(), s.data() + s.size()};
    }

    [[nodiscard]] std::array<int, 3> array3_i32_schem_to_std(
      MC_SCHEM_array3_i32 arr) noexcept {
      std::array<int, 3> result{};
      std::copy_n(arr.pos, 3, result.begin());
      return result;
    }

    [[nodiscard]] MC_SCHEM_array3_i32 array3_i32_std_to_schem(
      std::span<const int, 3> arr) noexcept {
      MC_SCHEM_array3_i32 result;
      std::copy_n(arr.begin(), 3, result.pos);
      return result;
    }

    void deep_swap(MC_SCHEM_string *a, MC_SCHEM_string *b) {
      MC_SCHEM_swap_string(a, b);
    }

    void deep_swap(MC_SCHEM_nbt_value *a, MC_SCHEM_nbt_value *b) {
      MC_SCHEM_swap_nbt(a, b);
    }

    void deep_swap(MC_SCHEM_block *a, MC_SCHEM_block *b) {
      MC_SCHEM_swap_block(a, b);
    }

    void deep_swap(MC_SCHEM_entity *a, MC_SCHEM_entity *b) {
      MC_SCHEM_swap_entity(a, b);
    }

    void deep_swap(MC_SCHEM_block_entity *a, MC_SCHEM_block_entity *b) {
      MC_SCHEM_swap_block_entity(a, b);
    }

    void deep_swap(MC_SCHEM_pending_tick *a, MC_SCHEM_pending_tick *b) {
      MC_SCHEM_swap_pending_tick(a, b);
    }

    void deep_swap(MC_SCHEM_error *a, MC_SCHEM_error *b) {
      MC_SCHEM_swap_error(a, b);
    }

    void deep_swap(MC_SCHEM_region *a, MC_SCHEM_region *b) {
      MC_SCHEM_swap_region(a, b);
    }

    void deep_swap(MC_SCHEM_schematic *a, MC_SCHEM_schematic *b) {
      MC_SCHEM_swap_schem(a, b);
    }

    template <typename handle_t>
    class wrapper {
     protected:
      handle_t handle{nullptr};

     public:
      using handle_type = handle_t;

      wrapper() = delete;

      explicit wrapper(handle_t p) : handle{p} {}

      wrapper(const wrapper &) = delete;

      wrapper(wrapper &&src) noexcept { std::swap(this->handle, src.handle); }

      wrapper &operator=(const wrapper &) = delete;

      wrapper &operator=(wrapper &&src) noexcept {
        std::swap(this->handle, src.handle);
      }

      [[nodiscard]] handle_t unwrap_handle() noexcept { return this->handle; }

      [[nodiscard]] const handle_t unwrap_handle() const noexcept {
        return this->handle;
      }

      void swap_handle(wrapper &another) noexcept {
        std::swap(this->handle, another.handle);
      }

      void deep_swap(wrapper &another) noexcept {
        deep_swap(this->handle, another.handle);
      }

      void reset_handle(handle_t ptr) noexcept { this->handle = ptr; }
    };


    template<class cpp_type, class c_array_view>
    [[nodiscard]] std::vector<cpp_type> c_view_to_vector(c_array_view view) noexcept {
      using handle_t = std::decay_t<decltype(view.begin)>;
      std::vector<cpp_type> result{};
      result.reserve(view.end - view.begin);
      for (auto ptr = view.begin; ptr < view.end; ptr++) {
        result.emplace_back(ptr);
      }
      return result;
    }


    class deleter {
     public:
      static void operator()(MC_SCHEM_block *s) noexcept {
        if (s == nullptr) return;
        MC_SCHEM_block_box box{s};
        MC_SCHEM_release_block(&box);
      }

      static void operator()(MC_SCHEM_nbt_value *v) noexcept {
        if (v == nullptr) return;
        MC_SCHEM_nbt_value_box box{v};
        MC_SCHEM_release_nbt(&box);
      }

      static void operator()(MC_SCHEM_entity *v) noexcept {
        if (v == nullptr) return;
        MC_SCHEM_entity_box box{v};
        MC_SCHEM_release_entity(&box);
      }

      static void operator()(MC_SCHEM_block_entity *v) noexcept {
        if (v == nullptr) return;
        MC_SCHEM_block_entity_box box{v};
        MC_SCHEM_release_block_entity(&box);
      }

      static void operator()(MC_SCHEM_pending_tick *v) noexcept {
        if (v == nullptr) return;
        MC_SCHEM_pending_tick_box box{v};
        MC_SCHEM_release_pending_tick(&box);
      }

      static void operator()(MC_SCHEM_error *v) noexcept {
        if (v == nullptr) return;
        MC_SCHEM_error_box box{v};
        MC_SCHEM_release_error(&box);
      }

      static void operator()(MC_SCHEM_region *r) noexcept {
        if (r == nullptr) return;
        MC_SCHEM_region_box box{r};
        MC_SCHEM_release_region(&box);
      }

      static void operator()(MC_SCHEM_schematic *s) noexcept {
        if (s == nullptr) return;
        MC_SCHEM_schematic_box box{s};
        MC_SCHEM_release_schem(&box);
      }

      //      void operator()(MC_SCHEM_map_ref *m) const noexcept {
      //        MC_SCHEM_map_box box{m};
      //      }
    };

    template <typename content_t, typename c_box_t>
    class box {
     public:
      using handle_t = typename content_t::handle_type;
      static_assert(std::is_same_v<handle_t, decltype(c_box_t{nullptr}.ptr)>);

     protected:
      content_t content{nullptr};

      handle_t handle() noexcept { return this->content.unwrap_handle(); }

      [[nodiscard]] const handle_t *handle() const noexcept {
        return this->content.unwrap_handle();
      }

     public:
      box() = default;

      box(const box &) = delete;

      box(box &&src) noexcept { this->content.swap_handle(src.content); }

      explicit box(c_box_t &&src) : content{src.ptr} { src.ptr = nullptr; }

      ~box() {
        if (this->handle() != nullptr) {
          deleter{}(this->handle());
        }
      }

      explicit operator bool() const noexcept {
        return this->handle() != nullptr;
      }

      content_t *operator->() noexcept { return &this->content; }

      const content_t *operator->() const noexcept { return &this->content; }
    };

    class error : public wrapper<MC_SCHEM_error *> {
     public:
      error() = delete;

      error(const error &) = delete;

      explicit error(MC_SCHEM_error *handle)
        : wrapper<MC_SCHEM_error *>{handle} {}

      void to_string(std::string &dest) const noexcept {
        dest.resize(1024);
        while (true) {
          const auto cap = dest.size();
          size_t len = 0;
          MC_SCHEM_error_to_string(this->handle, dest.data(), cap, &len);
          if (cap >= len) {
            dest.resize(len);
            break;
          }
          // size not enough
          dest.resize(cap * 2);
        }
        if (dest.back() == '\0') {
          dest.pop_back();
        }
      }

      [[nodiscard]] std::string to_string() const noexcept {
        std::string result;
        this->to_string(result);
        return result;
      }
    };

    using error_box = box<error, MC_SCHEM_error_box>;

  }  // namespace detail

  class error : public std::runtime_error {
   protected:
    detail::error_box content;

   public:
    error() = delete;

    error(const error &) = delete;

    error(error &&) = default;

    explicit error(detail::error_box &&box)
      : std::runtime_error{""}, content{std::move(box)} {
      static_cast<std::runtime_error &>(*this) =
        std::runtime_error{this->content->to_string()};
    }

    explicit error(MC_SCHEM_error_box &&box)
      : error{detail::error_box{std::move(box)}} {
      assert(this->content->unwrap_handle() != nullptr);
    }
  };

  class rust_string : public detail::wrapper<MC_SCHEM_string *> {
   public:
    rust_string() = delete;

    explicit rust_string(MC_SCHEM_string *handle)
      : detail::wrapper<MC_SCHEM_string *>(handle) {}

    explicit operator std::string_view() const noexcept {
      auto schem_sv = MC_SCHEM_string_unwrap(this->handle);
      return detail::string_view_schem_to_std(schem_sv);
    }

    explicit operator MC_SCHEM_string_view() const noexcept {
      return MC_SCHEM_string_unwrap(this->handle);
    }

    void reset(std::string_view str) noexcept {
      auto schem_sv = detail::string_view_std_to_schem(str);
      MC_SCHEM_string_set(this->handle, schem_sv);
    }
  };

  namespace detail {
    template <map_key_type key_e, typename key_t, map_value_type value_e,
              typename value_t>
    class map_wrapper {
     public:
      using key_ref_type =
        std::conditional_t<key_e == map_key_type::string, std::string_view,
                           std::span<const int, 3>>;

     protected:
      MC_SCHEM_map_ref map_ref{{0, 0}};

     public:
      map_wrapper() = delete;

      explicit map_wrapper(MC_SCHEM_map_ref handel) : map_ref{handel} {
        assert(MC_SCHEM_map_get_key_type(&handel) ==
               static_cast<MC_SCHEM_map_key_type>(key_e));
        assert(MC_SCHEM_map_get_value_type(&handel) ==
               static_cast<MC_SCHEM_map_value_type>(value_e));
      }

      map_wrapper(const map_wrapper &) = delete;

      map_wrapper(map_wrapper &&b) noexcept {
        std::swap(this->map_ref, b.map_ref);
      }

      ~map_wrapper() = default;

      static MC_SCHEM_key_wrapper wrap_key(key_ref_type key) noexcept {
        MC_SCHEM_key_wrapper kw;
        if constexpr (key_e == map_key_type::string) {
          kw.string = string_view_std_to_schem(key);
        } else {
          memcpy(reinterpret_cast<void *>(kw.pos), key.data(), key.size());
        }
        return kw;
      }

      static key_ref_type unwrap_key(MC_SCHEM_key_wrapper key) noexcept {
        if constexpr (key_e == map_key_type::string) {
          return string_view_schem_to_std(key.string);
        } else {
          return key.pos;
        }
      }

      static MC_SCHEM_value_wrapper wrap_value(const value_t &value) noexcept {
        MC_SCHEM_value_wrapper vw;
        if constexpr (value_e == map_value_type::string) {
          vw.string = value.unwrap_handle();
        } else if constexpr (value_e == map_value_type::block_entity) {
          vw.block_entity = value.unwrap_handle();
        } else if constexpr (value_e == map_value_type::nbt) {
          vw.nbt = value.unwrap_handle();
        } else {
          vw.pending_tick_list = value.unwrap_handle();
        }
        return vw;
      }

      static auto unwrap_value(MC_SCHEM_value_wrapper vw) noexcept {
        if constexpr (value_e == map_value_type::string) {
          return vw.string;
        } else if constexpr (value_e == map_value_type::block_entity) {
          return vw.block_entity;
        } else if constexpr (value_e == map_value_type::nbt) {
          return vw.nbt;
        } else {
          return vw.pending_tick_list;
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
        return MC_SCHEM_map_contains_key(
          &this->map_ref, static_cast<MC_SCHEM_map_key_type>(key_e), &k);
      }

      // using foreach_fun_const = void (*)(size_t index, key_ref_type key,
      // const value_t &value);
      using foreach_fun_const_with_data = void (*)(size_t index,
                                                   key_ref_type key,
                                                   const value_t &value,
                                                   void *custom_data);
      // using foreach_fun_mut = void (*)(size_t index, key_ref_type key,
      // value_t &value);
      using foreach_fun_mut_with_data = void (*)(size_t index, key_ref_type key,
                                                 value_t &value,
                                                 void *custom_data);

     protected:
      struct callback_data_mut {
        foreach_fun_mut_with_data original_fun;
        void *original_custom_data;
      };

      static void fun_wrap_mut(size_t index, MC_SCHEM_key_wrapper key,
                               MC_SCHEM_value_wrapper value,
                               void *callback_data_p) {
        const callback_data_mut *data =
          reinterpret_cast<callback_data_mut *>(callback_data_p);
        auto k = unwrap_key(key);
        auto v = unwrap_value(value);
        data->original_fun(index, k, v, data->original_custom_data);
      }

      struct callback_data_const {
        foreach_fun_const_with_data original_fun;
        void *original_custom_data;
      };

      static void fun_wrap_const(size_t index, MC_SCHEM_key_wrapper key,
                                 MC_SCHEM_value_wrapper value,
                                 void *callback_data_p) {
        const callback_data_const *data =
          reinterpret_cast<callback_data_const *>(callback_data_p);
        auto k = unwrap_key(key);
        auto v = unwrap_value(value);
        data->original_fun(index, k, v, data->original_custom_data);
      }

     public:
      void foreach (foreach_fun_mut_with_data fun, void *custom_data) {
        callback_data_mut data{fun, custom_data};
        MC_SCHEM_map_foreach(&this->map_ref, fun_wrap_mut, &data);
      }

      void foreach (const std::function<void(size_t index, key_ref_type key,
                                             value_t &value)> &fun) {
        using stdfun_t = std::decay_t<decltype(fun)>;
        this->foreach (
          [](size_t idx, key_ref_type k, value_t &v, void *std_fun_p) {
            const stdfun_t &fun_p = *reinterpret_cast<stdfun_t *>(std_fun_p);
            fun_p(idx, k, v);
          },
          &fun);
      }

      void foreach (foreach_fun_const_with_data fun, void *custom_data) const {
        callback_data_const data{fun, custom_data};
        MC_SCHEM_map_foreach(&this->map_ref, fun_wrap_mut, &data);
      }

      void foreach (
        const std::function<void(size_t index, key_ref_type key,
                                 const value_t &value)> &fun) const {
        using stdfun_t = std::decay_t<decltype(fun)>;
        this->foreach (
          [](size_t idx, key_ref_type k, const value_t &v, void *std_fun_p) {
            const stdfun_t &fun_p = *reinterpret_cast<stdfun_t *>(std_fun_p);
            fun_p(idx, k, v);
          },
          &fun);
      }

     protected:
      [[nodiscard]] std::optional<value_t> impl_get(
        key_ref_type key) const noexcept {
        bool ok = false;
        auto k = wrap_key(key);
        auto val_union =
          MC_SCHEM_map_find(&this->map_ref, key_e, value_e, &k, &ok);
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

      [[nodiscard]] std::optional<const value_t> get(
        key_ref_type key) const noexcept {
        auto result = this->impl_get(key);
        if (result.has_value()) {
          return std::move(result.value());
        }
        return std::nullopt;
      }

      void insert(key_ref_type key, const value_t &value) noexcept {
        auto k = wrap_key(key);
        auto v = wrap_value(value);
        MC_SCHEM_map_insert(&this->map_ref, k, v);
      }

      // returns true if an element is remove, false if key doesn't exist
      bool remove(key_ref_type key) noexcept {
        auto k = wrap_key(key);
        bool ret = false;
        MC_SCHEM_map_remove(&this->map_ref, k, &ret);
        return ret;
      }

     public:
      template <bool is_const>
      class iterator_impl {
       protected:
        MC_SCHEM_map_iterator it;
        // std::optional<std::pair<key_t, value_t> > deref;

        explicit iterator_impl(MC_SCHEM_map_iterator it) : it{it} {}

        friend class map_wrapper;

       public:
        iterator_impl() = delete;

        [[nodiscard]] key_ref_type key() const noexcept {
          MC_SCHEM_iterator_deref_result deref =
            MC_SCHEM_map_iterator_deref(&this->it);
          assert(deref.has_value);
          if (!deref.has_value) {
            abort();
          }
          return unwrap_key(deref.key);
        }

        [[nodiscard]] std::conditional_t<is_const, const value_t, value_t>
        value() const noexcept {
          MC_SCHEM_iterator_deref_result deref =
            MC_SCHEM_map_iterator_deref(&this->it);
          assert(deref.has_value);
          if (!deref.has_value) {
            abort();
          }
          auto value = unwrap_value(deref.value);
          return value_t{value};
          //          if constexpr (value_e == map_value_type::string) {
          //            auto str = MC_SCHEM_string_unwrap(value);
          //            return string_view_schem_to_std(str);
          //          } else {
          //          }
        }

        iterator_impl &operator++() noexcept {
          MC_SCHEM_map_iterator_add(&this->it);
          return *this;
        }

        const iterator_impl operator++(int) noexcept {
          iterator_impl copy{*this};
          (*this)++;
          return copy;
        }

        [[nodiscard]] bool operator==(const iterator_impl &b) const noexcept {
          return MC_SCHEM_map_iterator_equal(&this->it, &b.it);
        }
      };

      using iterator = iterator_impl<false>;
      using const_iterator = iterator_impl<true>;

     protected:
      [[nodiscard]] MC_SCHEM_map_iterator impl_begin() const noexcept {
        bool ok = false;
        auto it = MC_SCHEM_map_iterator_first(
          &this->map_ref, static_cast<MC_SCHEM_map_key_type>(key_e),
          static_cast<MC_SCHEM_map_value_type>(value_e), &ok);
        assert(ok);
        return it;
      }

      [[nodiscard]] MC_SCHEM_map_iterator impl_end() const noexcept {
        bool ok = false;
        auto it = MC_SCHEM_map_iterator_end(
          &this->map_ref, static_cast<MC_SCHEM_map_key_type>(key_e),
          static_cast<MC_SCHEM_map_value_type>(value_e), &ok);
        assert(ok);
        return it;
      }

     public:
      [[nodiscard]] iterator begin() noexcept {
        return iterator{this->impl_begin()};
      }

      [[nodiscard]] iterator end() noexcept {
        return iterator{this->impl_end()};
      }

      [[nodiscard]] const_iterator begin() const noexcept {
        return this->cbegin();
      }

      [[nodiscard]] const_iterator end() const noexcept { return this->cend(); }

      [[nodiscard]] const_iterator cbegin() const noexcept {
        return const_iterator{this->impl_begin()};
      }

      [[nodiscard]] const_iterator cend() const noexcept {
        return const_iterator{this->impl_end()};
      }
    };
  }  // namespace detail

  class block : public detail::wrapper<MC_SCHEM_block *> {
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
    block() = delete;

    explicit block(MC_SCHEM_block *handle)
      : detail::wrapper<MC_SCHEM_block *>{handle} {}

    [[nodiscard]] std::string_view get_namespace() const noexcept {
      return detail::string_view_schem_to_std(
        MC_SCHEM_block_get_namespace(this->handle));
    }

    void set_namespace(std::string_view ns) noexcept {
      MC_SCHEM_block_set_namespace(this->handle,
                                   detail::string_view_std_to_schem(ns));
    }

    [[nodiscard]] std::string_view id() const noexcept {
      return detail::string_view_schem_to_std(
        MC_SCHEM_block_get_id(this->handle));
    }

    void set_id(std::string_view new_id) noexcept {
      MC_SCHEM_block_set_id(this->handle,
                            detail::string_view_std_to_schem(new_id));
    }

    using attribute_map_t =
      detail::map_wrapper<map_key_type::string, std::string_view,
                          map_value_type::string, rust_string>;

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
        MC_SCHEM_block_to_full_id(this->unwrap_handle(), dest.data(),
                                  dest.size(), &length);
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

    [[nodiscard]] std::string full_id() const noexcept {
      std::string result;
      this->full_id(result);
      return result;
    }

    using block_box_t = detail::box<mc_schem::block, MC_SCHEM_block_box>;

    static block_box_t create() noexcept {
      return block_box_t{MC_SCHEM_create_block()};
    }

    static std::expected<block_box_t, id_parse_error> parse_block(
      std::string_view full_id) noexcept {
      auto result = create();
      MC_SCHEM_block_id_parse_error error;

      const bool ok =
        MC_SCHEM_parse_block(detail::string_view_std_to_schem(full_id),
                             result->unwrap_handle(), &error);
      if (ok) {
        return std::move(result);
      }
      return std::unexpected(static_cast<id_parse_error>(error));
    }
  };

  class nbt : public detail::wrapper<MC_SCHEM_nbt_value *> {
   public:
    enum class tag_type : uint8_t {
      tag_byte = 1,
      tag_short = 2,
      tag_int = 3,
      tag_long = 4,
      tag_float = 5,
      tag_double = 6,
      tag_byte_array = 7,
      tag_string = 8,
      tag_list = 9,
      tag_compound = 10,
      tag_int_array = 11,
      tag_long_array = 12,
    };

    static std::string_view tag_type_to_string(tag_type t) noexcept {
      switch (t) {
        case tag_type::tag_byte:
          return "byte";
        case tag_type::tag_short:
          return "short";
        case tag_type::tag_int:
          return "int";
        case tag_type::tag_long:
          return "long";
        case tag_type::tag_float:
          return "float";
        case tag_type::tag_double:
          return "double";
        case tag_type::tag_byte_array:
          return "byte_array";
        case tag_type::tag_string:
          return "string";
        case tag_type::tag_list:
          return "list";
        case tag_type::tag_compound:
          return "compound";
        case tag_type::tag_int_array:
          return "int_array";
        case tag_type::tag_long_array:
          return "long_array";
      }
    }

   public:
    nbt() = delete;

    explicit nbt(MC_SCHEM_nbt_value *handle)
      : detail::wrapper<MC_SCHEM_nbt_value *>{handle} {}

    [[nodiscard]] tag_type type() const noexcept {
      return static_cast<enum tag_type>(MC_SCHEM_nbt_get_type(this->handle));
    }

    using compound_map_type =
      detail::map_wrapper<map_key_type::string, std::string_view,
                          map_value_type::nbt, nbt>;

    using variant_rep_const =
      std::variant<int8_t, int16_t, int32_t, int64_t, float, double,
                   std::span<const int8_t>, std::string_view,
                   std::span<const nbt>, const compound_map_type,
                   std::span<const int32_t>, std::span<const int64_t>>;

    using variant_rep_mut =
      std::variant<int8_t, int16_t, int32_t, int64_t, float, double,
                   std::span<int8_t>, std::span<char>, std::span<nbt>,
                   compound_map_type, std::span<int32_t>, std::span<int64_t>>;

    class nbt_unwrap_exception : public std::exception {
     protected:
      tag_type actual_type;
      tag_type expected_type;
      std::string what_str;

     public:
      nbt_unwrap_exception(tag_type actual, tag_type expected)
        : actual_type{actual}, expected_type{expected} {
        this->what_str =
          std::format("Trying to unwrap a {} nbt tag as {}",
                      tag_type_to_string(actual), tag_type_to_string(expected));
      }

      [[nodiscard]] const char *what() const noexcept override {
        return this->what_str.c_str();
      }
    };

    //    template<tag_type t>
    //    [[nodiscard]] auto get() const {
    //      constexpr int index = static_cast<int>(t) - 1;
    //      return std::get<index>(this->to_variant());
    //    }

    [[nodiscard]] int8_t as_byte() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_byte(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_byte};
      }
      return ret;
    }

    [[nodiscard]] int16_t as_short() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_short(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_short};
      }
      return ret;
    }

    [[nodiscard]] int32_t as_int() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_int(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_int};
      }
      return ret;
    }

    [[nodiscard]] int64_t as_long() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_long(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_long};
      }
      return ret;
    }

    [[nodiscard]] float as_float() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_float(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_float};
      }
      return ret;
    }

    [[nodiscard]] double as_double() const {
      bool ok = false;
      auto ret = MC_SCHEM_nbt_get_double(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_double};
      }
      return ret;
    }

    [[nodiscard]] std::string_view as_string() const {
      bool ok = false;
      auto schem_str = MC_SCHEM_nbt_get_string(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_string};
      }
      auto schem_sv = MC_SCHEM_string_unwrap(schem_str);
      return detail::string_view_schem_to_std(schem_sv);
    }

   protected:
    [[nodiscard]] std::span<int8_t> impl_as_byte_array() const {
      bool ok = false;
      auto arr_view = MC_SCHEM_nbt_get_byte_array(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_byte_array};
      }
      return std::span<int8_t>{arr_view.begin, arr_view.end};
    }

    [[nodiscard]] std::vector<nbt> impl_as_list() const {
      bool ok = false;
      auto arr_view = MC_SCHEM_nbt_get_list(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_list};
      }
      std::vector<nbt> result;
      result.reserve(arr_view.end - arr_view.begin);
      for (auto p = arr_view.begin; p < arr_view.end; p++) {
        result.emplace_back(p);
      }
      return result;
    }

    [[nodiscard]] compound_map_type impl_as_compound() const {
      bool ok = false;
      auto handle = MC_SCHEM_nbt_get_compound(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_compound};
      }
      return compound_map_type{handle};
    }

    [[nodiscard]] std::span<int32_t> impl_as_int_array() const {
      bool ok = false;
      auto arr_view = MC_SCHEM_nbt_get_int_array(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_int_array};
      }
      return std::span<int32_t>{arr_view.begin, arr_view.end};
    }

    [[nodiscard]] std::span<int64_t> impl_as_long_array() const {
      bool ok = false;
      auto arr_view = MC_SCHEM_nbt_get_long_array(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_long_array};
      }
      return std::span<int64_t>{arr_view.begin, arr_view.end};
    }

   public:
    [[nodiscard]] std::span<const int8_t> as_byte_array() const {
      return this->impl_as_byte_array();
    }

    [[nodiscard]] std::span<int8_t> as_byte_array() {
      return this->impl_as_byte_array();
    }

    [[nodiscard]] std::span<const int32_t> as_int_array() const {
      return this->impl_as_int_array();
    }

    [[nodiscard]] std::span<int32_t> as_int_array() {
      return this->impl_as_int_array();
    }

    [[nodiscard]] std::span<const int64_t> as_long_array() const {
      return this->impl_as_long_array();
    }

    [[nodiscard]] std::span<int64_t> as_long_array() {
      return this->impl_as_long_array();
    }

    [[nodiscard]] const std::vector<nbt> as_list() const {
      return this->impl_as_list();
    }

    [[nodiscard]] std::vector<nbt> as_list() { return this->impl_as_list(); }

    [[nodiscard]] const compound_map_type as_compound() const {
      return this->impl_as_compound();
    }

    [[nodiscard]] compound_map_type as_compound() {
      return this->impl_as_compound();
    }

   public:
    void set(int8_t v) noexcept { MC_SCHEM_nbt_set_byte(this->handle, v); }

    void set(int16_t v) noexcept { MC_SCHEM_nbt_set_short(this->handle, v); }

    void set(int32_t v) noexcept { MC_SCHEM_nbt_set_int(this->handle, v); }

    void set(int64_t v) noexcept { MC_SCHEM_nbt_set_long(this->handle, v); }

    void set(float v) noexcept { MC_SCHEM_nbt_set_float(this->handle, v); }

    void set(double v) noexcept { MC_SCHEM_nbt_set_double(this->handle, v); }

    void set(std::span<const int8_t> v) noexcept {
      auto *data = const_cast<int8_t *>(v.data());
      MC_SCHEM_nbt_set_byte_array(
        this->handle, MC_SCHEM_nbt_byte_array_view{data, data + v.size()});
    }

    void set(std::span<const int32_t> v) noexcept {
      auto *data = const_cast<int32_t *>(v.data());
      MC_SCHEM_nbt_set_int_array(
        this->handle, MC_SCHEM_nbt_int_array_view{data, data + v.size()});
    }

    void set(std::span<const int64_t> v) noexcept {
      auto *data = const_cast<int64_t *>(v.data());
      MC_SCHEM_nbt_set_long_array(
        this->handle, MC_SCHEM_nbt_long_array_view{data, data + v.size()});
    }

    void set(std::span<const nbt> v) noexcept {
      auto *data = const_cast<nbt *>(v.data());
      MC_SCHEM_nbt_set_list(
        this->handle,
        MC_SCHEM_nbt_list_view{
          reinterpret_cast<MC_SCHEM_nbt_value *>(data),
          reinterpret_cast<MC_SCHEM_nbt_value *>(data + v.size())});
    }

    void set(std::string_view v) noexcept {
      auto schem_sv = detail::string_view_std_to_schem(v);
      MC_SCHEM_nbt_set_string(this->handle, schem_sv);
    }

    template <class T>
    nbt &operator=(T src) noexcept {
      this->set(src);
      return *this;
    }

    static detail::box<nbt, MC_SCHEM_nbt_value_box> create() noexcept {
      MC_SCHEM_nbt_value_box box = MC_SCHEM_create_nbt();
      return detail::box<nbt, MC_SCHEM_nbt_value_box>{std::move(box)};
    }
  };

  class entity : public detail::wrapper<MC_SCHEM_entity *> {
   public:
    entity() = delete;

    entity(MC_SCHEM_entity *handle)
      : detail::wrapper<MC_SCHEM_entity *>{handle} {}

    [[nodiscard]] std::array<int, 3> block_pos() const noexcept {
      MC_SCHEM_array3_i32 pos = MC_SCHEM_entity_get_block_pos(this->handle);
      std::array<int, 3> ret{};
      for (size_t i = 0; i < 3; i++) {
        ret[i] = pos.pos[i];
      }
      return ret;
    }

    [[nodiscard]] std::array<double, 3> pos() const noexcept {
      MC_SCHEM_array3_f64 pos = MC_SCHEM_entity_get_pos(this->handle);
      std::array<double, 3> ret{};
      for (size_t i = 0; i < 3; i++) {
        ret[i] = pos.pos[i];
      }
      return ret;
    }

    void set_block_pos(std::span<const int, 3> pos) noexcept {
      MC_SCHEM_array3_i32 p;
      for (size_t i = 0; i < 3; i++) {
        p.pos[i] = pos[i];
      }
      MC_SCHEM_entity_set_block_pos(this->handle, p);
    }

    void set_pos(std::span<const double, 3> pos) noexcept {
      MC_SCHEM_array3_f64 p;
      for (size_t i = 0; i < 3; i++) {
        p.pos[i] = pos[i];
      }
      MC_SCHEM_entity_set_pos(this->handle, p);
    }

    using tag_nbt_map_t =
      detail::map_wrapper<map_key_type::string, std::string_view,
                          map_value_type::nbt, nbt>;

   protected:
    [[nodiscard]] tag_nbt_map_t impl_tags() const noexcept {
      return tag_nbt_map_t{MC_SCHEM_entity_get_tags(this->handle)};
    }

   public:
    [[nodiscard]] const tag_nbt_map_t tags() const noexcept {
      return this->impl_tags();
    }

    [[nodiscard]] tag_nbt_map_t tags() noexcept { return this->impl_tags(); }

    [[nodiscard]] static detail::box<entity, MC_SCHEM_entity_box>
    create() noexcept {
      return detail::box<entity, MC_SCHEM_entity_box>{MC_SCHEM_create_entity()};
    }
  };

  class block_entity : public detail::wrapper<MC_SCHEM_block_entity *> {
   public:
    block_entity() = delete;

    block_entity(MC_SCHEM_block_entity *handle)
      : detail::wrapper<MC_SCHEM_block_entity *>{handle} {}

    using tag_nbt_map_t =
      detail::map_wrapper<map_key_type::string, std::string_view,
                          map_value_type::nbt, nbt>;

   protected:
    [[nodiscard]] tag_nbt_map_t impl_tags() const noexcept {
      return tag_nbt_map_t{MC_SCHEM_block_entity_get_tags(this->handle)};
    }

   public:
    [[nodiscard]] const tag_nbt_map_t tags() const noexcept {
      return this->impl_tags();
    }

    [[nodiscard]] tag_nbt_map_t tags() noexcept { return this->impl_tags(); }

    [[nodiscard]] static detail::box<block_entity, MC_SCHEM_block_entity_box>
    create() noexcept {
      return detail::box<block_entity, MC_SCHEM_block_entity_box>{
        MC_SCHEM_create_block_entity()};
    }
  };

  class pending_tick : public detail::wrapper<MC_SCHEM_pending_tick *> {
   public:
    pending_tick() = delete;

    pending_tick(MC_SCHEM_pending_tick *handle)
      : detail::wrapper<MC_SCHEM_pending_tick *>{handle} {}

    enum class pending_tick_type : uint8_t {
      fluid = 0,
      block = 1,
    };

    [[nodiscard]] int32_t priority() const noexcept {
      return MC_SCHEM_pending_tick_get_priority(this->handle);
    }

    void set_priority(int32_t priority) noexcept {
      MC_SCHEM_pending_tick_set_priority(this->handle, priority);
    }

    [[nodiscard]] int64_t sub_tick() const noexcept {
      return MC_SCHEM_pending_tick_get_sub_tick(this->handle);
    }

    void set_sub_tick(int64_t sub_tick) noexcept {
      MC_SCHEM_pending_tick_set_sub_tick(this->handle, sub_tick);
    }

    [[nodiscard]] int32_t time() const noexcept {
      return MC_SCHEM_pending_tick_get_time(this->handle);
    }

    void set_time(int32_t time) noexcept {
      MC_SCHEM_pending_tick_set_time(this->handle, time);
    }

    [[nodiscard]] std::string_view id() const noexcept {
      return detail::string_view_schem_to_std(
        MC_SCHEM_pending_tick_get_id(this->handle));
    }

    [[nodiscard]] pending_tick_type type() const noexcept {
      return static_cast<pending_tick_type>(
        MC_SCHEM_pending_tick_get_type(this->handle));
    }

    void set_info(pending_tick_type type, std::string_view id) noexcept {
      const auto t = static_cast<MC_SCHEM_pending_tick_type>(type);
      MC_SCHEM_pending_tick_set_info(this->handle, t,
                                     detail::string_view_std_to_schem(id));
    }

    static detail::box<pending_tick, MC_SCHEM_pending_tick_box>
    create() noexcept {
      return detail::box<pending_tick, MC_SCHEM_pending_tick_box>{
        MC_SCHEM_create_pending_tick()};
    }
  };

  class region : public detail::wrapper<MC_SCHEM_region *> {
   public:
    region() = delete;

    region(const region &) = delete;

    region(region &&) = default;

    explicit region(MC_SCHEM_region *handle)
      : detail::wrapper<MC_SCHEM_region *>{handle} {}

    static detail::box<region, MC_SCHEM_region_box> create() noexcept {
      return detail::box<region, MC_SCHEM_region_box>{MC_SCHEM_create_region()};
    }

    [[nodiscard]] std::string_view name() const noexcept {
      return detail::string_view_schem_to_std(
        MC_SCHEM_region_get_name(this->handle));
    }

    void set_name(std::string_view name) noexcept {
      MC_SCHEM_region_set_name(this->handle,
                               detail::string_view_std_to_schem(name));
    }

    [[nodiscard]] std::array<int, 3> offset() const noexcept {
      auto offset = MC_SCHEM_region_get_offset(this->handle);
      return detail::array3_i32_schem_to_std(offset);
    }

    void set_offset(std::span<const int, 3> offset) noexcept {
      MC_SCHEM_region_set_offset(this->handle,
                                 detail::array3_i32_std_to_schem(offset));
    }

    [[nodiscard]] const std::vector<block> palette() const noexcept {
      size_t len = 0;
      auto pal = MC_SCHEM_region_get_palette(this->handle, &len);
      std::vector<block> result;
      result.reserve(len);
      for (size_t i = 0; i < len; i++) {
        result.push_back(block{pal + i});
      }
      return result;
    }

    void set_palette(std::span<const MC_SCHEM_block *> pal) noexcept {
      MC_SCHEM_region_set_palette(this->handle, pal.data(), pal.size());
    }

    void set_palette(std::span<const block> pal) noexcept {
      std::vector<const MC_SCHEM_block *> p;
      p.reserve(pal.size());
      for (auto &blk : pal) {
        p.push_back(blk.unwrap_handle());
      }
      this->set_palette(p);
    }

    using block_entity_map =
      detail::map_wrapper<map_key_type::pos_i32, std::span<const int, 3>,
                          map_value_type::block_entity, block_entity>;
    using pending_tick_map =
      detail::map_wrapper<map_key_type::pos_i32, std::span<const int, 3>,
        map_value_type::pending_tick, std::vector<pending_tick>>;

   protected:
    [[nodiscard]] block_entity_map impl_block_entities() const noexcept {
      return block_entity_map{MC_SCHEM_region_get_block_entities(this->handle)};
    }

    [[nodiscard]] pending_tick_map impl_pending_ticks() const noexcept {
      return pending_tick_map{MC_SCHEM_region_get_pending_ticks(this->handle)};
    }

    [[nodiscard]] std::span<uint16_t> impl_block_index_array() const noexcept {
      auto ptr = MC_SCHEM_region_get_block_index_array(this->handle);
      return {ptr, this->volume()};
    }

   public:
    [[nodiscard]] block_entity_map block_entities() noexcept {
      return this->impl_block_entities();
    }

    [[nodiscard]] const block_entity_map block_entities() const noexcept {
      return this->impl_block_entities();
    }

    [[nodiscard]] pending_tick_map pending_ticks() noexcept {
      return this->impl_pending_ticks();
    }

    [[nodiscard]] const pending_tick_map pending_ticks() const noexcept {
      return this->impl_pending_ticks();
    }

    [[nodiscard]] uint64_t volume() const noexcept {
      return MC_SCHEM_region_get_volume(this->handle);
    }

    [[nodiscard]] uint64_t total_blocks(bool include_air) const noexcept {
      return MC_SCHEM_region_get_total_blocks(this->handle, include_air);
    }

    [[nodiscard]] std::array<int, 3> shape() const noexcept {
      return detail::array3_i32_schem_to_std(
        MC_SCHEM_region_get_shape(this->handle));
    }

    void reshape(std::span<const int, 3> shape) noexcept {
      MC_SCHEM_region_reshape(this->handle,
                              detail::array3_i32_std_to_schem(shape));
    }

    [[nodiscard]] const block block_at(
      std::span<const int, 3> r_pos) const noexcept {
      auto ptr = MC_SCHEM_region_get_block(
        this->handle, detail::array3_i32_std_to_schem(r_pos));
      return block{const_cast<MC_SCHEM_block *>(ptr)};
    }

    [[nodiscard]] bool set_block_at(std::span<const int, 3> r_pos,
                                    const block &blk) noexcept {
      return MC_SCHEM_region_set_block(this->handle,
                                       detail::array3_i32_std_to_schem(r_pos),
                                       blk.unwrap_handle());
    }

    [[nodiscard]] uint16_t block_index_at(
      std::span<const int, 3> r_pos) const noexcept {
      return MC_SCHEM_region_get_block_index(
        this->handle, detail::array3_i32_std_to_schem(r_pos));
    }

    [[nodiscard]] bool set_block_index_at(std::span<const int, 3> r_pos,
                                          uint16_t block_index) noexcept {
      return MC_SCHEM_region_set_block_index(
        this->handle, detail::array3_i32_std_to_schem(r_pos), block_index);
    }

    [[nodiscard]] std::optional<uint16_t> block_index_of_air() const noexcept {
      bool ok = false;
      auto result = MC_SCHEM_region_get_block_index_of_air(this->handle, &ok);
      if (ok) {
        return result;
      }
      return std::nullopt;
    }

    [[nodiscard]] std::optional<uint16_t> block_index_of_structure_void()
      const noexcept {
      bool ok = false;
      auto result =
        MC_SCHEM_region_get_block_index_of_structure_void(this->handle, &ok);
      if (ok) {
        return result;
      }
      return std::nullopt;
    }

    [[nodiscard]] bool contains_coordinate(
      std::span<const int, 3> r_pos) const noexcept {
      return MC_SCHEM_region_contains_coordinate(
        this->handle, detail::array3_i32_std_to_schem(r_pos));
    }

    struct block_info {
      uint16_t block_index;
      const ::mc_schem::block block;
      block_entity block_entity;
      std::vector<pending_tick> pending_ticks;
    };

   protected:
    [[nodiscard]] block_info impl_block_info_at(
      std::span<const int, 3> r_pos) const noexcept {
      auto result = MC_SCHEM_region_get_block_info(
        this->handle, detail::array3_i32_std_to_schem(r_pos));
      return block_info{
        result.block_index,
        block{const_cast<MC_SCHEM_block *>(result.block)},
        block_entity{const_cast<MC_SCHEM_block_entity *>(result.block_entity)},
        detail::c_view_to_vector<pending_tick>(result.pending_ticks)
      };
    }

   public:
    [[nodiscard]] const block_info block_info_at(
      std::span<const int, 3> r_pos) const noexcept {
      return this->impl_block_info_at(r_pos);
    }

    [[nodiscard]] block_info block_info_at(
      std::span<const int, 3> r_pos) noexcept {
      return this->impl_block_info_at(r_pos);
    }

    void shrink_palette() {
      auto box = MC_SCHEM_region_shrink_palette(this->handle);
      if (box.ptr != nullptr) {
        throw error{std::move(box)};
      }
    }
  };

  enum class common_block : uint16_t {
    air = 0,
    structure_void = 1,
  };

  struct litematica_load_option {
    using c_type = MC_SCHEM_load_option_litematica;
    static_assert(sizeof(c_type) == 512);

    litematica_load_option()
      : litematica_load_option{MC_SCHEM_load_option_litematica_default()} {}

    explicit litematica_load_option(const c_type &) {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      c_type result{};
      return result;
    }
  };

  struct litematica_save_option {
    using c_type = MC_SCHEM_save_option_litematica;
    static_assert(sizeof(c_type) == 512);

    uint32_t compress_level;
    bool rename_duplicated_regions;

    explicit litematica_save_option(const c_type &src)
      : compress_level{src.compress_level},
        rename_duplicated_regions{src.rename_duplicated_regions} {}

    litematica_save_option()
      : litematica_save_option{MC_SCHEM_save_option_litematica_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      return c_type{
        this->compress_level,
        this->rename_duplicated_regions,
      };
    }
  };

  struct vanilla_structure_load_option {
    using c_type = MC_SCHEM_load_option_vanilla_structure;
    static_assert(sizeof(c_type) == 512);
    common_block background_block;

    explicit vanilla_structure_load_option(const c_type &src)
      : background_block{src.background_block} {}

    vanilla_structure_load_option()
      : vanilla_structure_load_option{
          MC_SCHEM_load_option_vanilla_structure_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      c_type result{static_cast<MC_SCHEM_common_block>(this->background_block)};
      return result;
    }
  };

  struct vanilla_structure_save_option {
    using c_type = MC_SCHEM_save_option_vanilla_structure;
    static_assert(sizeof(c_type) == 512);

    uint32_t compress_level;
    bool keep_air;

    explicit vanilla_structure_save_option(const c_type &src)
      : compress_level{src.compress_level}, keep_air{src.keep_air} {}

    vanilla_structure_save_option()
      : vanilla_structure_save_option{
          MC_SCHEM_save_option_vanilla_structure_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      return c_type{
        this->compress_level,
        this->keep_air,
      };
    }
  };

  struct world_edit_13_load_option {
    using c_type = MC_SCHEM_load_option_world_edit_13;
    static_assert(sizeof(c_type) == 512);

    explicit world_edit_13_load_option(const c_type &) {}

    world_edit_13_load_option()
      : world_edit_13_load_option{
          MC_SCHEM_load_option_world_edit_13_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      c_type result{};
      return result;
    }
  };

  struct world_edit_13_save_option {
    using c_type = MC_SCHEM_save_option_world_edit_13;
    static_assert(sizeof(c_type) == 512);

    uint32_t compress_level;
    common_block background_block;

    explicit world_edit_13_save_option(const c_type &src)
      : compress_level{src.compress_level},
        background_block{static_cast<common_block>(src.background_block)} {}

    world_edit_13_save_option()
      : world_edit_13_save_option{
          MC_SCHEM_save_option_world_edit_13_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      return c_type{
        this->compress_level,
        static_cast<MC_SCHEM_common_block>(this->background_block),
      };
    }
  };

  struct world_edit_12_load_option {
    int32_t data_version;
    //    bool fix_string_id_with_block_entity_data;
    //    bool discard_number_id_array;

    using c_type = MC_SCHEM_load_option_world_edit_12;

    explicit world_edit_12_load_option(const c_type &src)
      : data_version{src.data_version} {}

    world_edit_12_load_option()
      : world_edit_12_load_option{
          MC_SCHEM_load_option_world_edit_12_default()} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      c_type result{this->data_version};
      return result;
    }
  };

  namespace detail {
    template <typename T, typename c_option_t>
    [[nodiscard]] std::optional<T> parse_c_option(
      const c_option_t &src) noexcept {
      if (src.has_value) {
        return T{src.value};
      }
      return std::nullopt;
    }

    template <typename c_option_t>
    [[nodiscard]] std::optional<decltype(c_option_t{}.value)> parse_c_option(
      const c_option_t &src) noexcept {
      if (src.has_value) {
        return {src.value};
      }
      return std::nullopt;
    }

    [[nodiscard]] std::optional<std::string> parse_c_option(
      const MC_SCHEM_optional_string_view &src) noexcept {
      if (src.has_value) {
        return std::string{string_view_schem_to_std(src.value)};
      }
      return std::nullopt;
    }

    [[nodiscard]] std::optional<std::array<int, 3>> parse_c_option(
      const MC_SCHEM_optional_i32_array3 &src) noexcept {
      if (src.has_value) {
        return {array3_i32_schem_to_std(src.value)};
      }
      return std::nullopt;
    }

    [[nodiscard]] MC_SCHEM_optional_i32 make_c_option(
      const std::optional<int32_t> &src) noexcept {
      return {src.value_or(0), src.has_value()};
    }

    [[nodiscard]] MC_SCHEM_optional_i64 make_c_option(
      const std::optional<int64_t> &src) noexcept {
      return {src.value_or(0), src.has_value()};
    }

    [[nodiscard]] MC_SCHEM_optional_string_view make_c_option(
      const std::optional<std::string> &src) noexcept {
      if (src.has_value()) {
        return {string_view_std_to_schem(src.value()), true};
      }
      return {MC_SCHEM_string_view{nullptr, nullptr}, false};
    }

    [[nodiscard]] MC_SCHEM_optional_i32_array3 make_c_option(
      const std::optional<std::array<int32_t, 3>> &src) noexcept {
      if (src.has_value()) {
        return {array3_i32_std_to_schem(src.value()), true};
      }
      return {MC_SCHEM_array3_i32{{0, 0, 0}}, false};
    }
  }  // namespace detail

  struct metadata {
    using c_type = MC_SCHEM_schem_metadata_c_rep;
    static_assert(sizeof(c_type) == 1024);

    int32_t mc_data_version;
    int64_t time_created;
    int64_t time_modified;
    std::string author;
    std::string name;
    std::string description;
    // litematica-related
    int32_t litematica_version;
    std::optional<int32_t> litematica_subversion;

    // world edit 12&13 (.schem/.schematic) related
    int32_t schem_version;  // world edit schem
    std::array<int32_t, 3> schem_offset;
    std::optional<std::array<int32_t, 3>> schem_we_offset;

    // std::optional<int64_t> date;

    // world edit 12 related
    std::optional<std::string> schem_world_edit_version;
    std::optional<std::string> schem_editing_platform;
    std::optional<std::array<int32_t, 3>> schem_origin;
    std::string schem_material;  // Classic or Alpha

    explicit metadata(const c_type &src)
      : mc_data_version{src.mc_data_version},
        time_created{src.time_created},
        time_modified{src.time_modified},
        author{detail::string_view_schem_to_std(src.author)},
        name{detail::string_view_schem_to_std(src.name)},
        description{detail::string_view_schem_to_std(src.author)},
        litematica_version{src.litematica_version},
        litematica_subversion{
          detail::parse_c_option(src.litematica_subversion)},
        schem_version{src.schem_version},
        schem_offset{src.schem_offset[0], src.schem_offset[1],
                     src.schem_offset[2]},
        schem_we_offset{detail::parse_c_option(src.schem_we_offset)},
        // date{detail::parse_c_option(src.date)},
        schem_world_edit_version{
          detail::parse_c_option(src.schem_world_edit_version)},
        schem_editing_platform{
          detail::parse_c_option(src.schem_editing_platform)},
        schem_origin{detail::parse_c_option(src.schem_origin)},
        schem_material{detail::string_view_schem_to_std(src.schem_material)} {}

    [[nodiscard]] c_type to_c_type() const noexcept {
      return c_type{
        this->mc_data_version,
        this->time_created,
        this->time_modified,
        detail::string_view_std_to_schem(this->author),
        detail::string_view_std_to_schem(this->name),
        detail::string_view_std_to_schem(this->description),

        this->litematica_version,
        detail::make_c_option(this->litematica_subversion),

        this->schem_version,
        {this->schem_offset[0], this->schem_offset[1], this->schem_offset[2]},
        detail::make_c_option(this->schem_we_offset),

        // detail::make_c_option(this->date),

        detail::make_c_option(this->schem_world_edit_version),
        detail::make_c_option(this->schem_editing_platform),
        detail::make_c_option(this->schem_origin),
        detail::string_view_std_to_schem(this->schem_material)};
    }
  };

  class schematic : public detail::wrapper<MC_SCHEM_schematic *> {
   public:
    schematic() = delete;

    explicit schematic(MC_SCHEM_schematic *handle)
      : detail::wrapper<MC_SCHEM_schematic *>{handle} {}

    using schem_box = detail::box<schematic, MC_SCHEM_schematic_box>;

    [[nodiscard]] static schem_box create() noexcept {
      return schem_box{MC_SCHEM_create_schem()};
    }

    using load_result = std::expected<schem_box, error>;

    [[nodiscard]] static load_result c_result_to_load_result(
      MC_SCHEM_schem_load_result &&src) noexcept {
      if (src.error.ptr != nullptr) {
        return std::unexpected(error{std::move(src.error)});
      }
      return schem_box{std::move(src.schematic)};
    }

    using save_result = std::expected<void, error>;

    [[nodiscard]] static save_result c_error_box_to_save_result(
      MC_SCHEM_error_box &&ret) noexcept {
      if (ret.ptr != nullptr) {
        return std::unexpected(error{std::move(ret)});
      }
      return {};
    }

    [[nodiscard]] static MC_SCHEM_reader wrap_istream(
      std::istream &src) noexcept {
      MC_SCHEM_reader result;
      result.handle = reinterpret_cast<void *>(&src);
      result.read_fun = [](void *handle, uint8_t *buffer, size_t buffer_size,
                           bool *ok, char *error,
                           size_t error_capacity) noexcept -> size_t {
        auto *is = reinterpret_cast<std::istream *>(handle);
        size_t bytes = 0;
        try {
          bytes = is->readsome(reinterpret_cast<char *>(buffer),
                               static_cast<ptrdiff_t>(buffer_size));
        } catch (std::exception &e) {
          *ok = false;
          snprintf(error, error_capacity, "%s", e.what());
          return bytes;
        }

        *ok = true;
        return bytes;
      };
      return result;
    }

    [[nodiscard]] static MC_SCHEM_writer wrap_ostream(
      std::ostream &dst) noexcept {
      MC_SCHEM_writer result;
      result.handle = &dst;
      result.write_fun = [](void *handle, const uint8_t *buffer,
                            size_t buffer_size, bool *ok, char *error,
                            size_t error_capacity) noexcept -> size_t {
        auto *os = reinterpret_cast<std::ostream *>(handle);
        try {
          os->write(reinterpret_cast<const char *>(buffer), buffer_size);
        } catch (std::exception &e) {
          *ok = false;
          snprintf(error, error_capacity, "%s", e.what());
          return 0;
        }
        *ok = true;
        return buffer_size;
      };
      result.flush_fun = [](void *handle, bool *ok, char *error,
                            size_t error_capacity) noexcept {
        auto *os = reinterpret_cast<std::ostream *>(handle);
        try {
          os->flush();
        } catch (std::exception &e) {
          *ok = false;
          snprintf(error, error_capacity, "%s", e.what());
          return;
        }
        *ok = true;
      };
      return result;
    }

    // load litematica
    [[nodiscard]] static load_result load_litematica(
      std::string_view filename,
      const litematica_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto file_name = detail::string_view_std_to_schem(filename);
      auto result = MC_SCHEM_schem_load_litematica_file(file_name, &c_option);

      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_litematica(
      std::span<const uint8_t> bytes,
      const litematica_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_litematica_bytes(
        bytes.data(), bytes.size_bytes(), &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_litematica(
      MC_SCHEM_reader &reader, const litematica_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_litematica(reader, &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_litematica(
      std::istream &src, const litematica_load_option &option) noexcept {
      auto reader = wrap_istream(src);
      return load_litematica(reader, option);
    }

    // load vanilla_structure
    [[nodiscard]] static load_result load_vanilla_structure(
      std::string_view filename,
      const vanilla_structure_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto file_name = detail::string_view_std_to_schem(filename);
      auto result =
        MC_SCHEM_schem_load_vanilla_structure_file(file_name, &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_vanilla_structure(
      std::span<const uint8_t> bytes,
      const vanilla_structure_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_vanilla_structure_bytes(
        bytes.data(), bytes.size_bytes(), &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_vanilla_structure(
      MC_SCHEM_reader &reader,
      const vanilla_structure_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_vanilla_structure(reader, &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_vanilla_structure(
      std::istream &src, const vanilla_structure_load_option &option) noexcept {
      auto reader = wrap_istream(src);
      return load_vanilla_structure(reader, option);
    }

    // load world_edit_13
    [[nodiscard]] static load_result load_world_edit_13(
      std::string_view filename,
      const world_edit_13_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto file_name = detail::string_view_std_to_schem(filename);
      auto result =
        MC_SCHEM_schem_load_world_edit_13_file(file_name, &c_option);

      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_world_edit_13(
      std::span<const uint8_t> bytes,
      const world_edit_13_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_world_edit_13_bytes(
        bytes.data(), bytes.size_bytes(), &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_world_edit_13(
      MC_SCHEM_reader &reader,
      const world_edit_13_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_world_edit_13(reader, &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_world_edit_13(
      std::istream &src, const world_edit_13_load_option &option) noexcept {
      auto reader = wrap_istream(src);
      return load_world_edit_13(reader, option);
    }

    // load world_edit_12
    [[nodiscard]] static load_result load_world_edit_12(
      std::string_view filename,
      const world_edit_12_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto file_name = detail::string_view_std_to_schem(filename);
      auto result =
        MC_SCHEM_schem_load_world_edit_12_file(file_name, &c_option);

      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_world_edit_12(
      std::span<const uint8_t> bytes,
      const world_edit_12_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_world_edit_12_bytes(
        bytes.data(), bytes.size_bytes(), &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] static load_result load_world_edit_12(
      MC_SCHEM_reader &reader,
      const world_edit_12_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto result = MC_SCHEM_schem_load_world_edit_12(reader, &c_option);
      return c_result_to_load_result(std::move(result));
    }
    [[nodiscard]] static load_result load_world_edit_12(
      std::istream &src, const world_edit_12_load_option &option) noexcept {
      auto c_option = option.to_c_type();
      auto reader = wrap_istream(src);
      auto result = MC_SCHEM_schem_load_world_edit_12(reader, &c_option);
      return c_result_to_load_result(std::move(result));
    }

    [[nodiscard]] save_result save_litematica(
      std::string_view filename,
      const litematica_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto file = detail::string_view_std_to_schem(filename);
      auto error =
        MC_SCHEM_schem_save_litematica_file(this->handle, file, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_litematica(
      MC_SCHEM_writer &writer,
      const litematica_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto error =
        MC_SCHEM_schem_save_litematica(this->handle, writer, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_litematica(
      std::ostream &dest, const litematica_save_option &option) const noexcept {
      auto writer = wrap_ostream(dest);
      return this->save_litematica(writer, option);
    }

    [[nodiscard]] save_result save_vanilla_structure(
      std::string_view filename,
      const vanilla_structure_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto file = detail::string_view_std_to_schem(filename);
      auto error = MC_SCHEM_schem_save_vanilla_structure_file(this->handle,
                                                              file, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_vanilla_structure(
      MC_SCHEM_writer &writer,
      const vanilla_structure_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto error =
        MC_SCHEM_schem_save_vanilla_structure(this->handle, writer, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_vanilla_structure(
      std::ostream &dest,
      const vanilla_structure_save_option &option) const noexcept {
      auto writer = wrap_ostream(dest);
      return this->save_vanilla_structure(writer, option);
    }

    [[nodiscard]] save_result save_world_edit_13(
      std::string_view filename,
      const world_edit_13_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto file = detail::string_view_std_to_schem(filename);
      auto error =
        MC_SCHEM_schem_save_world_edit_13_file(this->handle, file, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_world_edit_13(
      MC_SCHEM_writer &writer,
      const world_edit_13_save_option &option) const noexcept {
      auto c_option = option.to_c_type();
      auto error =
        MC_SCHEM_schem_save_world_edit_13(this->handle, writer, &c_option);
      return c_error_box_to_save_result(std::move(error));
    }

    [[nodiscard]] save_result save_world_edit_13(
      std::ostream &dest,
      const world_edit_13_save_option &option) const noexcept {
      auto writer = wrap_ostream(dest);
      return this->save_world_edit_13(writer, option);
    }

    [[nodiscard]] ::mc_schem::metadata metadata() const noexcept {
      return ::mc_schem::metadata{MC_SCHEM_schem_get_metadata(this->handle)};
    }

    void set_metadata(const MC_SCHEM_schem_metadata_c_rep *c_md) noexcept {
      MC_SCHEM_schem_set_metadata(this->handle, c_md);
    }

    void set_metadata(const ::mc_schem::metadata &src) noexcept {
      auto c_md = src.to_c_type();
      this->set_metadata(&c_md);
    }

    [[nodiscard]] size_t num_regions() const noexcept {
      return MC_SCHEM_schem_get_region_num(this->handle);
    }

   protected:
    [[nodiscard]] std::vector<region> impl_regions() const noexcept {
      std::vector<region> result;
      const size_t num_regions = MC_SCHEM_schem_get_region_num(this->handle);
      result.reserve(num_regions);

      for (size_t i = 0; i < num_regions; i++) {
        auto reg_ptr = MC_SCHEM_schem_get_region(this->handle, i);
        result.emplace_back(reg_ptr);
      }
      return result;
    }

   public:
    [[nodiscard]] std::vector<region> regions() noexcept {
      return this->impl_regions();
    }

    [[nodiscard]] const std::vector<region> regions() const noexcept {
      return this->impl_regions();
    }

    using region_box = detail::box<region, MC_SCHEM_region_box>;

    [[nodiscard]] region_box take_region(size_t index) noexcept {
      return region_box{MC_SCHEM_schem_take_region(this->handle, index)};
    }

    void insert_region(const region &reg, size_t index) noexcept {
      MC_SCHEM_schem_insert_region_copy(this->handle, reg.unwrap_handle(),
                                        index);
    }

    void insert_region(region_box &&box, size_t index) noexcept {
      MC_SCHEM_region_box b{box->unwrap_handle()};
      MC_SCHEM_schem_insert_region_move(this->handle, &b, index);
    }

    void block_indices_at(std::span<const int, 3> pos,
                          std::vector<uint16_t> &dest) const noexcept {
      size_t size = 0;
      dest.resize(this->num_regions());
      MC_SCHEM_schem_get_block_indices_at(this->handle,
                                          detail::array3_i32_std_to_schem(pos),
                                          &size, dest.data(), dest.size());
      assert(size <= dest.size());
      dest.resize(size);
    }

    [[nodiscard]] std::vector<uint16_t> block_indices_at(
      std::span<const int, 3> pos) const noexcept {
      std::vector<uint16_t> result;
      this->block_indices_at(pos, result);
      return result;
    }

    void blocks_at(std::span<const int, 3> pos,
                   std::vector<const MC_SCHEM_block *> &dest) const noexcept {
      size_t size = 0;
      dest.resize(this->num_regions());
      MC_SCHEM_schem_get_blocks_at(this->handle,
                                   detail::array3_i32_std_to_schem(pos), &size,
                                   dest.data(), dest.size());
      assert(size <= dest.size());
      dest.resize(size);
    }

    [[nodiscard]] const std::vector<block> blocks_at(
      std::span<const int, 3> pos) const noexcept {
      std::vector<const MC_SCHEM_block *> temp;
      this->blocks_at(pos, temp);
      std::vector<block> result;
      result.reserve(temp.size());
      for (auto handle : temp) {
        result.emplace_back(const_cast<MC_SCHEM_block *>(handle));
      }
      return result;
    }

    [[nodiscard]] std::optional<uint16_t> first_block_index_at(
      std::span<const int, 3> pos) const noexcept {
      return detail::parse_c_option(MC_SCHEM_schem_get_first_block_index_at(
        this->handle, detail::array3_i32_std_to_schem(pos)));
    }

    [[nodiscard]] std::optional<const block> first_block_at(
      std::span<const int, 3> pos) const noexcept {
      auto blkp = MC_SCHEM_schem_get_first_block_at(
        this->handle, detail::array3_i32_std_to_schem(pos));
      if (blkp != nullptr) {
        return block{const_cast<MC_SCHEM_block *>(blkp)};
      }
      return std::nullopt;
    }

    [[nodiscard]] std::array<int, 3> shape() const noexcept {
      return detail::array3_i32_schem_to_std(
        MC_SCHEM_schem_get_shape(this->handle));
    }

    [[nodiscard]] uint64_t volume() const noexcept {
      return MC_SCHEM_schem_get_volume(this->handle);
    }

    [[nodiscard]] uint64_t total_blocks(bool include_air) const noexcept {
      return MC_SCHEM_schem_get_total_blocks(this->handle, include_air);
    }
  };

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
