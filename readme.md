# Img

### Collectively share photo and video albums at their original quality, completely sovereign

#### Img is lightweight, simple and super efficient even when uploading large files.

## How To Use It
![topic screenshot](https://github.com/jaybutera/img/blob/master/static/testimg.png)
#### Make a new Collection
You can start a new collection from any page with "New Collection" in the top left corner. This will take you to the homepage. Just make a name and upload some initial pictures/videos to start it.

![homepage](https://github.com/jaybutera/img/assets/2101658/55f04e3e-b444-4924-9d93-7f48fc463544)

#### Add photos to an existing collection
From the first screenshot you'll see the "Add Photos" button in the top-right corner.

## What is it?
Img is essentially an open wiki for media. Anybody can create a topic by adding photos to it. The topic is an easily accessible url by anyone that knows it exists. Multiple people can upload to a single topic. It's an easy way to share photos with friends around an event, like a trip you took together, or an idea like cool art you find.

Img also makes downloading the images easy so they aren't locked in the cloud. Raw images are served at an endpoint which other tools can access and build on top of. Such tools may provide better UI's for viewing and organizing photos, or display collections in a native app. Img stores images without compressing, the original quality is preserved.

## Why does this exist?
Currently there's no organized and secure way to share photos. The choices are to host on a cloud like Google Photos, but the photos are then compressed and difficult to get back off the cloud and it's also not easy to share an album and have people add to it. Group messaging apps like Telegram and Signal are also used, but photos are not organized and get buried under new messages. Photos from these apps are also only stored on people's phones and so may be lost. And again it's not easy to share with new people.

There are a few other opensource self-host photo softwares but they are complex and bulky. Generally using complex techniques that obfuscate your images like loading into SQL databases. The Img code is tiny. You can understand and hack on it easily. Your files are all stored in one directory. You can back them up, sync, use version control or whatever else might enhance your setup.

## How?
Upon hosting your own sovereign server at example.com, to start a collection around your trip to alaska with friends, just visit `example.com/alaska/new` and upload your photos. Visit `example.com/alaska` to see them.

## Cool features
All photos are stored by (named by their hash) in one directory. Each collection is represented with a json file which stores the version history of adds/removes/reorders. A collection is a list of image references and the display order can be rearranged just by changing the json list.

Adding and removing photos can easily be done in command line as img provides standard API endpoints for standard operations (add/remove/etc). Bulk uploads are one simple bash script away. This makes img a nice blend between user-friendly photo-sharing service and a minimal/robust solution for photo storage and archival.

## Future Features
Eventually it would be nice to have a @user tag option in the url. Collections under a user tag can be permissioned as configured by the user. A user can decide who can see a collection and who can add to it. However img is designed to be small and do one thing well. A feature like this would take some consideration.


## Dev Notes

- Session security is false and samesite none to test locally. Should be turned off for prod.
