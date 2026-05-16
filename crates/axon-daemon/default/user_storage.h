#ifndef USER_STORAGE_H
#define USER_STORAGE_H

struct user_record {
    int id;
    char name[64];
};

// [✅ LOCKED]
// ownership: caller_retains
int save_user(struct user_record* user);

#endif
