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

  }

  class rust_string : public detail::wrapper<MC_SCHEM_string> {
  public:
    rust_string() = delete;

    rust_string(MC_SCHEM_string *handle) : detail::wrapper<MC_SCHEM_string>(handle) {}

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

      map_wrapper(const map_wrapper &&) = delete;

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
    block() = delete;

    block(MC_SCHEM_block *handle) : detail::wrapper<MC_SCHEM_block>{handle} {}

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


} // namespace mc_schem


#endif // MC_SCHEM_MC_SCHEM_HPP
