#include <mysql.h>
// #include <service_sql.h>
#include <mysql/plugin_encryption.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>

// AES128-GCM 128-bit key
#define KEY_LEN 16

void drop_connection(MYSQL *mysql) { mysql_close(mysql); }

int check_for_errors(MYSQL *mysql)
{
  const char *emsg= mysql_error(mysql);
  uint32_t errno= mysql_errno(mysql);

  if ((!emsg || !strlen(emsg)) && !errno)
  {
    return 0;
  }
  else if (emsg)
  {
    fprintf(stderr, "ERROR %d: %s", errno, emsg);
  }
  else
  {
    fprintf(stderr, "ERROR %d", errno);
  }

  return -1;
}

MYSQL *mysql_do_init()
{
  fprintf(stderr, "DEBUG: mysql_do_init\n");

  MYSQL *mysql= mysql_init(NULL);

  if (!mysql)
  {
    fprintf(stderr, "ERROR: mysql_init failed\n");
    return NULL;
  }

  // Validate we are using an expected charset
  int charset= mysql_options(mysql, MYSQL_SET_CHARSET_NAME, "utf8mb4");

  if (charset != 0)
  {
    fprintf(stderr, "ERROR: charset not recognized\n");
    return NULL;
  }

  return mysql;
}

MYSQL *connect_local()
{
  fprintf(stderr, "DEBUG: connect_local\n");
  MYSQL *mysql= mysql_do_init();

  MYSQL *res= mysql_real_connect_local(mysql);

  if (check_for_errors(mysql))
  {
    return NULL;
  }

  if (!res)
  {
    fprintf(stderr, "ERROR: connect error, maybe already connected?\n");
    return NULL;
  }

  return mysql;
}

static unsigned int
get_latest_key_version(unsigned int key_id)
{
  fprintf(stderr, "DEBUG: get_latest_key_version\n");

  MYSQL *mysql= connect_local();
  if (!mysql)
  {
    return 1;
  }

  drop_connection(mysql);

  return 1;
}

static unsigned int
get_key(unsigned int key_id, unsigned int version,
        unsigned char* dstbuf, unsigned *buflen)
{
  fprintf(stderr, "DEBUG: get_key\n");

  MYSQL *mysql= connect_local();
  if (!mysql)
  {
    return 1;
  }

  if (dstbuf == NULL) {
    *buflen = KEY_LEN;
  } else {
    memset(dstbuf, 9, *buflen);
  }

  drop_connection(mysql);

  return 0;
}

static int example_keymgt_sql_service_init(void *p)
{
  fprintf(stderr, "DEBUG: example_keymgt_sql_service_init\n");

  MYSQL *mysql= connect_local();
  if (!mysql)
  {
    return 1;
  }

  drop_connection(mysql);

  return 0;
}

static int example_keymgt_sql_service_deinit(void *p) { return 0; }

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
