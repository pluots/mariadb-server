
#include "handler_bridge.h"

extern "C" handler*
ha_construct_bridge(handlerton *hton, TABLE_SHARE *table_args,
                    MEM_ROOT *mem_root, const handler_bridge_vt* vt) {
  return new (mem_root) handler_bridge(hton, table_args, vt);
}

// TODO: does this get called automatically?
extern "C" void ha_destroy_bridge(handler*bridge) {
  delete (handler_bridge*)bridge;
}
