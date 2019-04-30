@echo -off

if exist "fs0:driver.efi" then
    fs0:
endif

if exist "fs1:driver.efi" then
    fs1:
endif

if exist "fs2:driver.efi" then
    fs2:
endif

if exist "fs3:driver.efi" then
    fs3:
endif

if exist "fs4:driver.efi" then
    fs4:
endif

if exist "fs5:driver.efi" then
    fs5:
endif

if exist "fs6:driver.efi" then
    fs6:
endif

if exist "fs7:driver.efi" then
    fs7:
endif

if exist "fs8:driver.efi" then
    fs8:
endif

if exist "fs9:driver.efi" then
    fs9:
endif

if not exist "driver.efi" then
    echo "Did not find driver.efi"
    exit 1
endif

load driver.efi
exit
