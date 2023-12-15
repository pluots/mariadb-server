
#include "handler_bridge.h"

extern "C" handler*
ha_bridge_construct(handlerton *hton, TABLE_SHARE *table_args,
                    MEM_ROOT *mem_root, const handler_bridge_vt* vt) {
  return new (mem_root) handler_bridge(hton, table_args, mem_root, vt);
}

// TODO: does this get called automatically?
extern "C" void ha_bridge_destroy(handler*bridge) {
  delete (handler_bridge*)bridge;
}
