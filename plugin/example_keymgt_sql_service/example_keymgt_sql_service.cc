#include <my_global.h>
#include <my_pthread.h>
#include <mysql/plugin_encryption.h>
#include <my_crypt.h>
#include <string.h>

// AES128-GCM 128-bit key
#define KEY_LEN 16

static unsigned int
get_latest_key_version(unsigned int key_id)
{
  return 1;
}

static unsigned int
get_key(unsigned int key_id, unsigned int version,
        unsigned char* dstbuf, unsigned *buflen)
{
  if (dstbuf == NULL) {
    *buflen = KEY_LEN;
  } else {
    memset(dstbuf, 9, *buflen);
  }

  return 0;
}

static int example_keymgt_sql_service_init(void *p)
{
  return 0;
}

static int example_keymgt_sql_service_deinit(void *p)
{
  return 0;
}


struct st_mariadb_encryption example_keymgt_sql_service= {
  MariaDB_ENCRYPTION_INTERFACE_VERSION,
  get_latest_key_version,
  get_key,
  NULL,
  NULL,
  NULL,
  NULL,
  NULL,
};

/*
  Plugin library descriptor
*/
maria_declare_plugin(example_keymgt_sql_service)
{
  MariaDB_ENCRYPTION_PLUGIN,
  &example_keymgt_sql_service,
  "example_keymgt_sql_service",
  "Trevor",
  "Example keymgt plugin that uses sql service",
  PLUGIN_LICENSE_GPL,
  example_keymgt_sql_service_init,
  example_keymgt_sql_service_deinit,
  0x0100 /* 1.0 */,
  NULL,	/* status variables */
  NULL,	/* system variables */
  "1.0",
  MariaDB_PLUGIN_MATURITY_EXPERIMENTAL
}
maria_declare_plugin_end;
