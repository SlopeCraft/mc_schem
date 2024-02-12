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
#include <functional>
#include <exception>
#include <variant>
#include <format>
#include <array>

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

    template<typename handle_t>
    class wrapper {
    protected:
      handle_t handle{nullptr};
    public:
      using handle_type = handle_t;

      wrapper() = delete;

      wrapper(handle_t p) : handle{p} {}

      wrapper(const wrapper &) = delete;

      wrapper(wrapper &&src) {
        std::swap(this->handle, src.handle);
      }

      wrapper &operator=(const wrapper &) = delete;

      wrapper &operator=(wrapper &&src) noexcept {
        std::swap(this->handle, src.handle);
      }

      [[nodiscard]] handle_t unwrap_handle() noexcept {
        return this->handle;
      }

      [[nodiscard]] const handle_t unwrap_handle() const noexcept {
        return this->handle;
      }

      void swap(wrapper &another) noexcept {
        std::swap(this->handle, another.handle);
      }

      void reset_handle(handle_t ptr) noexcept {
        this->handle = ptr;
      }
    };


    class deleter {
    public:
      static void operator()(MC_SCHEM_block *s) noexcept {
        MC_SCHEM_block_box box{s};
        MC_SCHEM_release_block(&box);
      }

      static void operator()(MC_SCHEM_nbt_value *v) noexcept {
        MC_SCHEM_nbt_value_box box{v};
        MC_SCHEM_release_nbt(&box);
      }

      static void operator()(MC_SCHEM_entity *v) noexcept {
        MC_SCHEM_entity_box box{v};
        MC_SCHEM_release_entity(&box);
      }

//      void operator()(MC_SCHEM_map_ref *m) const noexcept {
//        MC_SCHEM_map_box box{m};
//      }
    };

    template<typename content_t, typename c_box_t>
    class box {
    public:
      using handle_t = typename content_t::handle_type;
      static_assert(std::is_same_v<handle_t,
        decltype(c_box_t{nullptr}.ptr)>);
    protected:
      content_t content{nullptr};

      handle_t handle() noexcept {
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

  }

  class rust_string : public detail::wrapper<MC_SCHEM_string *> {
  public:
    rust_string() = delete;

    rust_string(MC_SCHEM_string *handle) : detail::wrapper<MC_SCHEM_string *>(handle) {}

    operator std::string_view() const noexcept {
      auto schem_sv = MC_SCHEM_string_unwrap(this->handle);
      return detail::string_view_schem_to_std(schem_sv);
    }

    operator MC_SCHEM_string_view() const noexcept {
      return MC_SCHEM_string_unwrap(this->handle);
    }

    void reset(std::string_view str) noexcept {
      auto schem_sv = detail::string_view_std_to_schem(str);
      MC_SCHEM_string_set(this->handle, schem_sv);
    }
  };

  namespace detail {
    template<map_key_type key_e, typename key_t, map_value_type value_e,
      typename value_t>
    class map_wrapper {
    public:
      using key_ref_type = std::conditional_t<key_e == map_key_type::string, std::string_view, std::span<const int, 3>>;

    protected:
      MC_SCHEM_map_ref map_ref;

    public:
      map_wrapper() = delete;

      map_wrapper(MC_SCHEM_map_ref handel) : map_ref{handel} {
        assert(MC_SCHEM_map_get_key_type(&handel) == static_cast<MC_SCHEM_map_key_type>(key_e));
        assert(MC_SCHEM_map_get_value_type(&handel) == static_cast<MC_SCHEM_map_value_type>(value_e));
      }

      map_wrapper(const map_wrapper &) = delete;

      map_wrapper(map_wrapper &&b) {
        std::swap(this->map_ref, b.map_ref);
      }

      ~map_wrapper() = default;

      static MC_SCHEM_key_wrapper wrap_key(key_ref_type key) noexcept {
        MC_SCHEM_key_wrapper kw;
        if constexpr (key_e == map_key_type::string) {
          kw.string = string_view_std_to_schem(key);
        } else {
          memcpy(reinterpret_cast<void *>( kw.pos), key.data(), key.size());
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
          vw.pending_tick = value.unwrap_handle();
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

      //using foreach_fun_const = void (*)(size_t index, key_ref_type key, const value_t &value);
      using foreach_fun_const_with_data = void (*)(size_t index, key_ref_type key, const value_t &value,
                                                   void *custom_data);
      //using foreach_fun_mut = void (*)(size_t index, key_ref_type key, value_t &value);
      using foreach_fun_mut_with_data = void (*)(size_t index, key_ref_type key, value_t &value, void *custom_data);

    protected:
      struct callback_data_mut {
        foreach_fun_mut_with_data original_fun;
        void *original_custom_data;
      };

      static void fun_wrap_mut(size_t index,
                               MC_SCHEM_key_wrapper key,
                               MC_SCHEM_value_wrapper value,
                               void *callback_data_p) {
        const callback_data_mut *data = reinterpret_cast<callback_data_mut *>(callback_data_p);
        auto k = unwrap_key(key);
        auto v = unwrap_value(value);
        data->original_fun(index, k, v, data->original_custom_data);
      }

      struct callback_data_const {
        foreach_fun_const_with_data original_fun;
        void *original_custom_data;
      };

      static void fun_wrap_const(size_t index,
                                 MC_SCHEM_key_wrapper key,
                                 MC_SCHEM_value_wrapper value,
                                 void *callback_data_p) {
        const callback_data_const *data = reinterpret_cast<callback_data_const *>(callback_data_p);
        auto k = unwrap_key(key);
        auto v = unwrap_value(value);
        data->original_fun(index, k, v, data->original_custom_data);
      }

    public:
      void foreach(foreach_fun_mut_with_data fun, void *custom_data) {
        callback_data_mut data{fun, custom_data};
        MC_SCHEM_map_foreach(&this->map_ref, fun_wrap_mut, &data);
      }

      void foreach(const std::function<void(size_t index, key_ref_type key, value_t &value)> &fun) {
        using stdfun_t = std::decay_t<decltype(fun)>;
        this->foreach([](size_t idx, key_ref_type k, value_t &v, void *std_fun_p) {
          const stdfun_t &fun_p = *reinterpret_cast<stdfun_t *>(std_fun_p);
          fun_p(idx, k, v);
        }, &fun);
      }

      void foreach(foreach_fun_const_with_data fun, void *custom_data) const {
        callback_data_const data{fun, custom_data};
        MC_SCHEM_map_foreach(&this->map_ref, fun_wrap_mut, &data);
      }

      void foreach(const std::function<void(size_t index, key_ref_type key, const value_t &value)> &fun) const {
        using stdfun_t = std::decay_t<decltype(fun)>;
        this->foreach([](size_t idx, key_ref_type k, const value_t &v, void *std_fun_p) {
          const stdfun_t &fun_p = *reinterpret_cast<stdfun_t *>(std_fun_p);
          fun_p(idx, k, v);
        }, &fun);
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
      template<bool is_const>
      class iterator_impl {
      protected:
        MC_SCHEM_map_iterator it;
        //std::optional<std::pair<key_t, value_t> > deref;

        explicit iterator_impl(MC_SCHEM_map_iterator it) : it{it} {

        }

        friend class map_wrapper;

      public:
        iterator_impl() = delete;

        const key_ref_type key() const noexcept {
          MC_SCHEM_iterator_deref_result deref = MC_SCHEM_map_iterator_deref(&this->it);
          assert(deref.has_value);
          if (!deref.has_value) {
            abort();
          }
          return unwrap_key(deref.key);
        }

        std::conditional_t<is_const, const value_t, value_t> value() const noexcept {
          MC_SCHEM_iterator_deref_result deref = MC_SCHEM_map_iterator_deref(&this->it);
          assert(deref.has_value);
          if (!deref.has_value) {
            abort();
          }
          auto value = unwrap_value(deref.value);
          return value;
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

        iterator_impl operator++(int) noexcept {
          iterator_impl copy{*this};
          (*this)++;
          return copy;
        }

        bool operator==(const iterator_impl &b) const noexcept {
          return MC_SCHEM_map_iterator_equal(&this->it, &b.it);
        }

      };

      using iterator = iterator_impl<false>;
      using const_iterator = iterator_impl<true>;

    protected:

      MC_SCHEM_map_iterator impl_begin() const noexcept {
        bool ok = false;
        auto it = MC_SCHEM_map_iterator_first(&this->map_ref,
                                              static_cast<MC_SCHEM_map_key_type>(key_e),
                                              static_cast<MC_SCHEM_map_value_type>(value_e), &ok);
        assert(ok);
        return it;
      }

      MC_SCHEM_map_iterator impl_end() const noexcept {
        bool ok = false;
        auto it = MC_SCHEM_map_iterator_end(&this->map_ref,
                                            static_cast<MC_SCHEM_map_key_type>(key_e),
                                            static_cast<MC_SCHEM_map_value_type>(value_e), &ok);
        assert(ok);
        return it;
      }

    public:
      iterator begin() noexcept {
        return iterator{this->impl_begin()};
      }

      iterator end() noexcept {
        return iterator{this->impl_end()};
      }

      const_iterator begin() const noexcept {
        return this->cbegin();
      }

      const_iterator end() const noexcept {
        return this->cend();
      }

      const_iterator cbegin() const noexcept {
        return const_iterator{this->impl_begin()};
      }

      const_iterator cend() const noexcept {
        return const_iterator{this->impl_end()};
      }
    };
  }

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

    block(MC_SCHEM_block *handle) : detail::wrapper<MC_SCHEM_block *>{handle} {}

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

    using attribute_map_t = detail::map_wrapper<map_key_type::string, std::string_view, map_value_type::string, rust_string>;
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

    nbt(MC_SCHEM_nbt_value *handle) : detail::wrapper<MC_SCHEM_nbt_value *>{handle} {}

    tag_type type() const noexcept {
      return static_cast<enum tag_type>(MC_SCHEM_nbt_get_type(this->handle));
    }

    using compound_map_type = detail::map_wrapper<map_key_type::string, std::string_view, map_value_type::nbt, nbt>;

    using variant_rep_const = std::variant<int8_t, int16_t, int32_t, int64_t,
      float, double,
      std::span<const int8_t>, std::string_view, std::span<const nbt>,
      const compound_map_type,
      std::span<const int32_t>, std::span<const int64_t>
    >;

    using variant_rep_mut = std::variant<int8_t, int16_t, int32_t, int64_t,
      float, double,
      std::span<int8_t>, std::span<char>, std::span<nbt>,
      compound_map_type,
      std::span<int32_t>, std::span<int64_t>
    >;

    class nbt_unwrap_exception : public std::exception {
    protected:
      tag_type actual_type;
      tag_type expected_type;
      std::string what_str;
    public:
      nbt_unwrap_exception(tag_type actual, tag_type expected) : actual_type{actual},
                                                                 expected_type{expected} {
        this->what_str = std::format("Trying to unwrap a {} nbt tag as {}",
                                     tag_type_to_string(actual),
                                     tag_type_to_string(expected));
      }

      const char *what() const noexcept override {
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

    [[nodiscard]] std::span<nbt> impl_as_list() const {
      bool ok = false;
      auto arr_view = MC_SCHEM_nbt_get_list(this->handle, &ok);
      if (!ok) {
        throw nbt_unwrap_exception{this->type(), tag_type::tag_list};
      }
      return std::span<nbt>{reinterpret_cast<nbt *>(arr_view.begin),
                            reinterpret_cast<nbt *>(arr_view.end)};
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

    [[nodiscard]] std::span<const nbt> as_list() const {
      return this->impl_as_list();
    }

    [[nodiscard]] std::span<nbt> as_list() {
      return this->impl_as_list();
    }

    [[nodiscard]] const compound_map_type as_compound() const {
      return this->impl_as_compound();
    }

    [[nodiscard]] compound_map_type as_compound() {
      return this->impl_as_compound();
    }

  public:

    void set(int8_t v) noexcept {
      MC_SCHEM_nbt_set_byte(this->handle, v);
    }

    void set(int16_t v) noexcept {
      MC_SCHEM_nbt_set_short(this->handle, v);
    }

    void set(int32_t v) noexcept {
      MC_SCHEM_nbt_set_int(this->handle, v);
    }

    void set(int64_t v) noexcept {
      MC_SCHEM_nbt_set_long(this->handle, v);
    }

    void set(float v) noexcept {
      MC_SCHEM_nbt_set_float(this->handle, v);
    }

    void set(double v) noexcept {
      MC_SCHEM_nbt_set_double(this->handle, v);
    }

    void set(std::span<const int8_t> v) noexcept {
      auto *data = const_cast<int8_t *>(v.data());
      MC_SCHEM_nbt_set_byte_array(this->handle,
                                  MC_SCHEM_nbt_byte_array_view{data, data + v.size()});
    }

    void set(std::span<const int32_t> v) noexcept {
      auto *data = const_cast<int32_t *>(v.data());
      MC_SCHEM_nbt_set_int_array(this->handle,
                                 MC_SCHEM_nbt_int_array_view{data, data + v.size()});
    }

    void set(std::span<const int64_t> v) noexcept {
      auto *data = const_cast<int64_t *>(v.data());
      MC_SCHEM_nbt_set_long_array(this->handle,
                                  MC_SCHEM_nbt_long_array_view{data, data + v.size()});
    }


    void set(std::span<const nbt> v) noexcept {
      auto *data = const_cast<nbt *>(v.data());
      MC_SCHEM_nbt_set_list(this->handle,
                            MC_SCHEM_nbt_list_view{reinterpret_cast<MC_SCHEM_nbt_value *>(data),
                                                   reinterpret_cast<MC_SCHEM_nbt_value *>(data + v.size())});
    }

    void set(std::string_view v) noexcept {
      auto schem_sv = detail::string_view_std_to_schem(v);
      MC_SCHEM_nbt_set_string(this->handle, schem_sv);
    }

    template<class T>
    nbt &operator=(T src) noexcept {
      this->set(src);
    }

    static detail::box<nbt, MC_SCHEM_nbt_value_box> create() noexcept {
      MC_SCHEM_nbt_value_box box = MC_SCHEM_create_nbt();
      return detail::box<nbt, MC_SCHEM_nbt_value_box>{std::move(box)};
    }

  };

  class entity : public detail::wrapper<MC_SCHEM_entity *> {
  public:
    entity() = delete;

    entity(MC_SCHEM_entity *handle) : detail::wrapper<MC_SCHEM_entity *>{handle} {}

    [[nodiscard]] std::array<int, 3> block_pos() const noexcept {
      MC_SCHEM_pos_i32 pos = MC_SCHEM_entity_get_block_pos(this->handle);
      std::array<int, 3> ret;
      for (size_t i = 0; i < 3; i++) {
        ret[i] = pos.pos[i];
      }
      return ret;
    }

    [[nodiscard]] std::array<double, 3> pos() const noexcept {
      MC_SCHEM_pos_f64 pos = MC_SCHEM_entity_get_pos(this->handle);
      std::array<double, 3> ret;
      for (size_t i = 0; i < 3; i++) {
        ret[i] = pos.pos[i];
      }
      return ret;
    }

    void set_block_pos(std::span<const int, 3> pos) noexcept {
      MC_SCHEM_pos_i32 p;
      for (size_t i = 0; i < 3; i++) {
        p.pos[i] = pos[i];
      }
      MC_SCHEM_entity_set_block_pos(this->handle, p);
    }

    void set_pos(std::span<const double, 3> pos) noexcept {
      MC_SCHEM_pos_f64 p;
      for (size_t i = 0; i < 3; i++) {
        p.pos[i] = pos[i];
      }
      MC_SCHEM_entity_set_pos(this->handle, p);
    }

    using tag_nbt_map_t = detail::map_wrapper<map_key_type::string, std::string_view, map_value_type::nbt, nbt>;
  protected:
    [[nodiscard]] tag_nbt_map_t impl_tags() const noexcept {
      return {MC_SCHEM_entity_get_tags(this->handle)};
    }

  public:
    [[nodiscard]] const tag_nbt_map_t tags() const noexcept {
      return this->impl_tags();
    }

    [[nodiscard]] tag_nbt_map_t tags() noexcept {
      return this->impl_tags();
    }

    [[nodiscard]]static detail::box<entity, MC_SCHEM_entity_box> create() noexcept {
      return detail::box<entity, MC_SCHEM_entity_box>{MC_SCHEM_create_entity()};
    }
  };

  class block_entity : public detail::wrapper<MC_SCHEM_block_entity *> {
  public:
    block_entity() = delete;

    block_entity(MC_SCHEM_block_entity *handle) : detail::wrapper<MC_SCHEM_block_entity *>{handle} {}

  };

  class region : public detail::wrapper<MC_SCHEM_region *> {
  public:
    region() = delete;

    region(MC_SCHEM_region *handle) : detail::wrapper<MC_SCHEM_region *>{handle} {}

  };

  class schem : public detail::wrapper<MC_SCHEM_schem *> {
  public:
    schem() = delete;

    schem(MC_SCHEM_schem *handle) : detail::wrapper<MC_SCHEM_schem *>{handle} {}

  };


} // namespace mc_schem


#endif // MC_SCHEM_MC_SCHEM_HPP
