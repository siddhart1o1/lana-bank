locals {
  apis = [
    "iam.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "compute.googleapis.com",
    "container.googleapis.com",
    "monitoring.googleapis.com",
    "stackdriver.googleapis.com",
    "servicenetworking.googleapis.com",
    "sqladmin.googleapis.com",
    "bigqueryconnection.googleapis.com",
    "maps-backend.googleapis.com",
    "apikeys.googleapis.com",
    "datamigration.googleapis.com",
    "secretmanager.googleapis.com"
  ]
}

resource "google_project_service" "service" {
  for_each           = toset(local.apis)
  project            = local.project
  service            = each.key
  disable_on_destroy = false
}

