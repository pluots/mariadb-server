/** @file handler_bridge.h

    @brief
  This file provides a bridge class that allows using a storage engine via a C
  API, rather than C++. Essentially this is needed to construct the needed
  vtables.
*/

#pragma once

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wreturn-type-c-linkage"
#endif

#include "my_global.h"
#include "handler.h"

class handler_bridge;

/**
  A C representation of the handler class.

  For now all function pointers must be nonnull. We could change this to check
  for null then call the parent function if so at some point.
*/
typedef struct handler_bridge_vt {
  void (*constructor)(handler_bridge*, handlerton*, MEM_ROOT*, TABLE_SHARE*);
  void (*destructor)(handler_bridge*);
  const char* (*index_type)(handler_bridge*, uint);
  ulonglong  (*table_flags)(const handler_bridge*);
  ulong (*index_flags)(const handler_bridge*, uint, uint, bool);
  uint (*max_supported_record_length)(const handler_bridge*);
  uint (*max_supported_keys)(const handler_bridge*);
  uint (*max_supported_key_parts)(const handler_bridge*);
  uint (*max_supported_key_length)(const handler_bridge*);
  IO_AND_CPU_COST (*scan_time)(handler_bridge*);
  IO_AND_CPU_COST (*keyread_time)(handler_bridge*, uint, ulong, ha_rows, ulonglong);
  IO_AND_CPU_COST (*rnd_pos_time)(handler_bridge*, ha_rows);
  int (*open)(handler_bridge*, const char*, int, uint);
  int (*close)(handler_bridge*);
  int (*write_row)(handler_bridge*, const uchar*);
  int (*update_row)(handler_bridge*, const uchar*, const uchar*);
  int (*delete_row)(handler_bridge*, const uchar*);
  int (*index_read_map)(handler_bridge*, uchar*, const uchar*, key_part_map,
                        enum ha_rkey_function);
  int (*index_next)(handler_bridge*, uchar*);
  int (*index_prev)(handler_bridge*, uchar*);
  int (*index_first)(handler_bridge*, uchar*);
  int (*index_last)(handler_bridge*, uchar*);
  int (*rnd_init)(handler_bridge*, bool );
  int (*rnd_end)(handler_bridge*);
  int (*rnd_next)(handler_bridge*, uchar*);
  int (*rnd_pos)(handler_bridge*, uchar*, uchar*);
  void (*position)(handler_bridge*, const uchar*);
  int (*info)(handler_bridge*, uint);
  int (*extra)(handler_bridge*, enum ha_extra_function);
  int (*external_lock)(handler_bridge*, THD*, int);
  int (*delete_all_rows)(handler_bridge*);
  ha_rows (*records_in_range)(handler_bridge*, uint, const key_range*,
                              const key_range*, page_range*);
  int (*delete_table)(handler_bridge*, const char*);
  int (*create)(handler_bridge*, const char*, TABLE*,  HA_CREATE_INFO*);
  enum_alter_inplace_result
  (*check_if_supported_inplace_alter)(handler_bridge*, TABLE*, Alter_inplace_info*);
  THR_LOCK_DATA** (*store_lock)(handler_bridge*, THD*, THR_LOCK_DATA**, enum thr_lock_type);
} handler_bridge_vt;


/** Wrapper that can expose a C vtable as a C++ class */
class handler_bridge final: public handler {
public:
  /** The vtable that we defer to for all method calls */
  const handler_bridge_vt *const vt;
  /** Storage for anything needed. Should only be touched by the C API, not this class. */
  void *data;
  /** Just a convenience point for a Rust type ID */
  uint8_t type_id[16];
  
  handler_bridge(handlerton *hton, TABLE_SHARE *table_arg,
                 MEM_ROOT *mem_root, const handler_bridge_vt *const vt)
    :handler(hton, table_arg),
    vt(vt)
  {
    vt->constructor(this, hton, mem_root, table_arg);
  }
  
  virtual ~handler_bridge() {
    vt->destructor(this);
  }
  
  const char *index_type(uint inx) { return vt->index_type(this, inx); } 
  ulonglong table_flags() const { return vt->table_flags(this); }
  ulong index_flags(uint inx, uint part, bool all_parts) const {
    return vt->index_flags(this, inx, part, all_parts);
  }
  uint max_supported_record_length() const { return vt->max_supported_record_length(this); }
  uint max_supported_keys() const { return vt->max_supported_keys(this); }
  uint max_supported_key_parts() const { return vt->max_supported_key_parts(this); }
  uint max_supported_key_length() const { return vt->max_supported_key_length(this); }
  virtual IO_AND_CPU_COST scan_time() { return vt->scan_time(this); }
  virtual IO_AND_CPU_COST keyread_time(uint index, ulong ranges, ha_rows rows,
                                       ulonglong blocks) {
    return vt->keyread_time(this, index, ranges, rows, blocks);
  }
  virtual IO_AND_CPU_COST rnd_pos_time(ha_rows rows) { return vt->rnd_pos_time(this, rows); }
  int open(const char *name, int mode, uint test_if_locked) {
    return vt->open(this, name, mode, test_if_locked);
  }
  int close(void) { return vt->close(this); }
  int write_row(const uchar *buf) { return vt->write_row(this, buf); }
  int update_row(const uchar *old_data, const uchar *new_data) { return vt->update_row(this, old_data, new_data); }
  int delete_row(const uchar *buf) { return vt->delete_row(this, buf); }
  int index_read_map(uchar *buf, const uchar *key,
                     key_part_map keypart_map, enum ha_rkey_function find_flag) {
    return vt->index_read_map(this, buf, key, keypart_map, find_flag);
  }
  int index_next(uchar *buf) { return vt->index_next(this, buf); }
  int index_prev(uchar *buf) { return vt->index_prev(this, buf); }
  int index_first(uchar *buf) { return vt->index_first(this, buf); }
  int index_last(uchar *buf) { return vt->index_last(this, buf); }
  int rnd_init(bool scan) { return vt->rnd_init(this, scan); }
  int rnd_end() { return vt->rnd_end(this); }
  int rnd_next(uchar *buf) { return vt->rnd_next(this, buf); }
  int rnd_pos(uchar *buf, uchar *pos) { return vt->rnd_pos(this, buf, pos); }
  void position(const uchar *record) { return vt->position(this, record); }
  int info(uint flag) { return vt->info(this, flag); }
  int extra(enum ha_extra_function operation) { return vt->extra(this, operation); }
  int external_lock(THD *thd, int lock_type) { return vt->external_lock(this, thd, lock_type); }
  int delete_all_rows(void) { return vt->delete_all_rows(this); }
  ha_rows records_in_range(uint inx, const key_range *min_key, const key_range *max_key,
                           page_range *pages) {
    return vt->records_in_range(this, inx, min_key, max_key, pages);
  }
  int delete_table(const char *from) { return vt->delete_table(this, from); }
  int create(const char *name, TABLE *form,  HA_CREATE_INFO *create_info) {
    return vt->create(this, name, form, create_info);
  }
  enum_alter_inplace_result
  check_if_supported_inplace_alter(TABLE* altered_table, Alter_inplace_info* ha_alter_info) {
    return vt->check_if_supported_inplace_alter(this, altered_table, ha_alter_info);
  }
  THR_LOCK_DATA **store_lock(THD *thd, THR_LOCK_DATA **to, enum thr_lock_type lock_type) {
    return vt->store_lock(this, thd, to, lock_type);
  }
};

/**
  A builder that will create the C++ class from a C vtable. This is used to create a
  `handler` from a `handlerton`.
 */
extern "C" handler*
ha_bridge_construct(handlerton*, TABLE_SHARE*, MEM_ROOT*, const handler_bridge_vt*);

/** Destroy a `handler_bridge` */
extern "C" void ha_bridge_destroy(handler*);
