// ddldump prints the ent schema's authoritative PostgreSQL DDL (the
// Atlas auto-migration plan) to stdout without touching any database.
package main

import (
	"context"
	"entgo.io/ent/dialect"
	"fmt"
	"os"

	sqlschema "entgo.io/ent/dialect/sql/schema"
	"ariga.io/atlas/sql/migrate"

	_ "github.com/lib/pq"

	"go-wind-admin/app/admin/service/internal/data/ent"
	_ "go-wind-admin/app/admin/service/internal/data/ent/runtime"
)

func main() {
	// Open lazily; replay mode + hook interception never connects.
	client, err := ent.Open("postgres", "host=127.0.0.1 port=1 sslmode=disable user=x dbname=x password=x")
	if err != nil {
		fmt.Fprintln(os.Stderr, "open:", err)
		os.Exit(1)
	}
	defer client.Close()

	err = client.Schema.Create(
		context.Background(),
		sqlschema.WithMigrationMode(sqlschema.ModeReplay),
		sqlschema.WithForeignKeys(true),
		sqlschema.WithApplyHook(func(next sqlschema.Applier) sqlschema.Applier {
			return sqlschema.ApplyFunc(func(ctx context.Context, conn dialect.ExecQuerier, plan *migrate.Plan) error {
				for _, c := range plan.Changes {
					fmt.Println(c.Cmd + ";")
				}
				return nil // swallow: never execute
			})
		}),
	)
	if err != nil {
		fmt.Fprintln(os.Stderr, "plan:", err)
		os.Exit(1)
	}
}
