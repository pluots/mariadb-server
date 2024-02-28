/* SPDX-License-Identifier: GPL-2.0 */

#include "my_global.h"

#include "bufcursor.h"
#include "my_dbug.h"

#include <stdarg.h>
#include <stddef.h>
#include <string.h>

/** Create a new cursor at an existing buffer. There is no need to free this
 * object */
bufcursor bcurs_new(char *start, size_t capacity)
{
  bufcursor ret= {
      .pos= start,
      .end= start + capacity,
  };
  *ret.pos= 0;
  return ret;
}

/** The number of bytes remaining in the cursor's buffer. */
size_t bcurs_spare_capacity(const bufcursor *curs)
{
  DBUG_ASSERT(curs->end >= curs->pos && "cursor is in an invalid state");
  return (size_t) (curs->end - curs->pos);
}

/** Assert if there are not at least `len` bytes in the buffer, return the
 * number of remaining bytes
 */
size_t bcurs_ensure_spare_cap(const bufcursor *curs, size_t len)
{
  size_t remaining= bcurs_spare_capacity(curs);

  if (unlikely(remaining < len))
  {
    fprintf(stderr, "not enough space in the cursor; need %zu actual %zu\n",
            len, remaining);
    DBUG_ASSERT(0);
  }
  return remaining;
}

/** Write a format string to the cursor, wrapping snprintf internally
 *
 * Returns the number of bytes written, asserts on error.
 */
size_t bcurs_write(bufcursor *curs, const char *format, ...)
{
  va_list args;
  size_t remaining= bcurs_spare_capacity(curs);
  int res= -1;

  va_start(args, format);
  res= vsnprintf(curs->pos, remaining, format, args);
  DBUG_ASSERT(res >= 0 && (size_t) res <= remaining &&
              "encoding or OOB error");
  curs->pos+= res;
  va_end(args);
  return (size_t) res;
}

/** Write a string to the cursor and return the start of the appended string
 * (`strcat`)
 */
char *bcurs_write_str(bufcursor *curs, const char *s)
{
  size_t len= strlen(s);
  return bcurs_write_bytes(curs, s, len);
}

/** Copy a byte buffer to this cursor */
char *bcurs_write_bytes(bufcursor *curs, const char *s, size_t len)
{
  char *ret= curs->pos;
  bcurs_ensure_spare_cap(curs, len + 1);
  memcpy(curs->pos, s, len);
  curs->pos+= len;
  *curs->pos= '\0';
  return ret;
}

/** Write a single character to the string */
void bcurs_write_char(bufcursor *curs, char c)
{
  bcurs_ensure_spare_cap(curs, 2);
  *curs->pos= c;
  ++(curs->pos);
  *curs->pos= '\0';
}
