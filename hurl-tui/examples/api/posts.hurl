# Get all posts
GET https://jsonplaceholder.typicode.com/posts
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$" count == 100
jsonpath "$[0].userId" exists
jsonpath "$[0].title" exists


# Get single post
GET https://jsonplaceholder.typicode.com/posts/1
Accept: application/json

HTTP 200
[Asserts]
jsonpath "$.id" == 1
jsonpath "$.userId" == 1
jsonpath "$.title" isString
jsonpath "$.body" isString


# Create a new post
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json
{
  "title": "Hello World",
  "body": "This is a test post",
  "userId": 1
}

HTTP 201
[Asserts]
jsonpath "$.id" exists
jsonpath "$.title" == "Hello World"
