## kin
KaiOS webapp installer(root need)

Nokia 8000 4G with kaios 2.5.4.1 cannot jailbreak because of the webapp installation API has been removed.

So this tools use root to extract webapp data into **/data/local/webapps**

support two zip formats:
- contains application.zip
- contains manifest.webapp

### shortcoming
Now we need restart b2g to let system reload app list. So you have to watch boot animation once again.

### usage
```shell
# push kin into device
adb push kin /data/local/tmp/
# push webapp
adb shell app.zip /sdcard/

# into shell
adb shell
# add executable permission
chmod +x /data/local/tmp/kin
# install
/data/local/tmp/kin /sdcard/app.zip

# restart b2g
stop b2g&&start b2g
```