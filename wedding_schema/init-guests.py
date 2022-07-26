#!/bin/python3

import subprocess
import csv

def insert(sql):
    command = ['psql', '-v','ON_ERROR_STOP=1', '--username','<<<user>>>', '--dbname', '<<<password>>>']
    subprocess.run(command, input=sql, text=True, check=True)

def gen_invite_sql(uid, email, name, guests):
    password="<<<guest_password>>>"
    invite_sql = f"INSERT INTO public.invite (\"uid\", \"password\", \"email\", \"names\") VALUES ({uid}, '{password}', '{email}', '{name.strip()}');"

    guests_sql = [f"INSERT INTO public.guest (\"invite_uid\", \"name\")                 VALUES ({uid}, '{guest.strip()}');" for guest in guests if guest]

    return [invite_sql] + guests_sql

def main():
    sqls = []
    with open('invites.csv', newline='') as invite_file:
        for line in csv.reader(invite_file, delimiter=',', quotechar='"'):
            email, uid, name, *guests = line
            guests = [guest.strip() for guest in guests if guest]

            if not uid:
                continue
            
            #print()
            #print(uid, email)
            #print(name, guests)

            sqls += gen_invite_sql(uid,email,name,guests)
    

    sql = '\n'.join(sqls)

    sql = f"begin transaction; {sql}; commit;"

    insert(sql)

    print(done)

if __name__ == '__main__':
    main()
