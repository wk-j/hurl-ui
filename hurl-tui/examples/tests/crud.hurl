# CRUD operations test

# CREATE
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json
{
  "title": "Test Post",
  "body": "Test Body",
  "userId": 1
}

HTTP 201
[Captures]
post_id: jsonpath "$.id"
[Asserts]
jsonpath "$.title" == "Test Post"


# READ
GET https://jsonplaceholder.typicode.com/posts/1

HTTP 200
[Asserts]
jsonpath "$.id" == 1


# UPDATE
PUT https://jsonplaceholder.typicode.com/posts/1
Content-Type: application/json
{
  "id": 1,
  "title": "Updated Title",
  "body": "Updated Body",
  "userId": 1
}

HTTP 200
[Asserts]
jsonpath "$.title" == "Updated Title"


# DELETE
DELETE https://jsonplaceholder.typicode.com/posts/1

HTTP 200
